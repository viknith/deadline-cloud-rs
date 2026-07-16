# Session Cache

## Overview

The session cache is a process-wide store that avoids re-creating AWS SDK
clients on every API call. It holds a base SDK config (built from the
user's AWS profile), a cached account ID (from STS GetCallerIdentity),
and a map of queue-scoped configs (for DCM users accessing S3 or
CloudWatch via queue role assumption).

The base config is keyed by profile name — if the user switches profiles,
the cache is invalidated and rebuilt. Queue configs are keyed by
`(farm_id, queue_id)` and invalidated alongside the base session.

## Account ID Caching

The account ID is resolved once via STS `GetCallerIdentity` (with a 2s
timeout) and cached in the session. Subsequent `deadline_client()` calls
reuse the cached value without additional STS calls.

Three states: not yet resolved → resolved to `Some("123...")` → resolved
to `None` (STS unavailable/timed out). Cleared on session invalidation.

This matches the Python behavior where the boto3 session identity was
cached via `@lru_cache`. The account ID is used only for telemetry
enrichment — failure to resolve it is non-fatal.

## SessionContext

Tracks caller identity for User-Agent enrichment:
- `submitter_name`, `submitter_version` — set by GUI/DCC plugins via FFI
- `cli_command_name` — set before each CLI command dispatch

Format: `app/deadline-client#<version> submitter/<name>#<ver> cli-command/<cmd>`

## Queue Credential Provider

The queue credential provider implements the SDK's `ProvideCredentials`
trait. It calls `AssumeQueueRoleForUser` to get temporary credentials,
and the SDK automatically refreshes when they expire.

Error messages include actionable guidance:
- Throttling → "Please retry the operation later"
- Internal error → error message from service (no retry guidance)
- Other → "Contact your administrator"

## Profile Resolution

Session functions take `profile: Option<&str>` directly — callers extract
the profile from config before calling. The sentinel values `"(default)"`,
`"default"`, and `""` all map to `None` (default credential chain). Any
other value becomes a named profile. The helper
`session::resolve_profile_name(&config)` performs this extraction.

## Endpoint Override

Reads `AWS_ENDPOINT_URL_DEADLINE` and `AWS_ENDPOINT_URL_STS` environment
variables. Applied when building service clients. Used by tests to point
at the wiremock stub server.

The Deadline SDK prepends `management.` or `scheduling.` to the endpoint
hostname (Smithy host prefix). In tests, the stub server receives requests
at `management.localhost:PORT`. The `.localhost` TLD resolves to 127.0.0.1
per RFC 6761.

## Invalidation

The cache is cleared on logout and on explicit refresh. The next API call
triggers a full credential re-resolution.

## Proxy and CA Bundle

Two config settings — `settings.https_proxy` and `settings.ca_bundle` —
allow corporate environments to route all Deadline Cloud API calls through
an HTTP(S) proxy and/or verify TLS against a custom CA certificate bundle,
without setting process-wide environment variables that leak into other
programs.

### Settings

| Setting | Type | Default | Purpose |
|---------|------|---------|---------|
| `settings.https_proxy` | String (URL) | `""` (disabled) | Proxy URL (e.g. `http://proxy.corp:8080`) |
| `settings.ca_bundle` | Path | `""` (disabled) | PEM file for TLS certificate verification |

An empty value means "disabled — use environment variables / SDK defaults."

### Resolution

Settings are resolved from the Deadline Cloud config (`~/.deadline/config`)
at session/client construction time:

1. Read `settings.https_proxy` — if non-empty after trimming, use it as
   the proxy URL.
2. Read `settings.ca_bundle` — if non-empty after trimming, expand `~` to
   the user's home directory (the setting has `is_path: true`), and use
   the result as the CA bundle path.

When a caller supplies an explicit `&IniConfig`, proxy/CA are resolved
from that config. When no config is supplied, the on-disk default is read.

### Application

Both settings are applied at the **HTTP connector level** when building
AWS SDK clients. They affect all clients built by the session module:

- Base `DeadlineClient` (API calls)
- `StsClient` (GetCallerIdentity for telemetry)
- CloudWatch Logs clients (job monitoring, log retrieval)
- Queue-user-scoped configs (S3 upload/download via queue credentials)

Concretely:

- **Proxy:** Configure the HTTP client's proxy settings with the URL
  applied to both `http` and `https` schemes (so the proxy is honored
  regardless of endpoint scheme).
- **CA bundle:** Configure the HTTP client's TLS trust store to load
  certificates from the specified PEM file path.

### Behavioral Rules

1. **Empty = disabled.** When the setting is empty (default), no proxy/CA
   override is applied. The SDK's normal behavior applies (respects
   `HTTPS_PROXY`, `AWS_CA_BUNDLE` env vars, system trust store).

2. **Setting wins over env var.** When a non-empty value is configured,
   it takes precedence over the corresponding environment variable for
   all Deadline-created clients.

3. **First-write-wins for cached sessions.** Sessions are cached by
   profile (see [Concurrency](#concurrency)). The proxy/CA from the first
   config that builds a given cached session are "baked in." A subsequent
   call with different proxy/CA values for the same cached session logs a
   warning but does not override — clients already built from the session
   captured the original values at construction time. Call
   `invalidate_session_cache()` (force refresh) to pick up changed values.

4. **Both settings or neither.** The two settings are independent. A user
   can set only a proxy (no custom CA), only a CA bundle (no proxy), or
   both together. A common scenario is both: a TLS-intercepting proxy that
   re-signs traffic with a corporate CA.

5. **No path validation.** A non-existent `ca_bundle` path is not
   validated at config-read time. The TLS handshake fails at connection
   time with an appropriate error from the HTTP client.

6. **No URL validation.** A malformed `https_proxy` value is not validated
   at config-read time. Connection attempts fail with an HTTP client error.

### Scope Boundaries

- **In scope:** All Deadline-created AWS SDK clients (Deadline, STS,
  CloudWatch Logs, S3 via queue credentials).
- **Out of scope:** Fire-and-forget telemetry STS client (best-effort,
  does not block user operations). Job attachments S3 clients owned by
  external packages (tracked separately).
- **GUI config dialog:** Not updated. Settings are manageable via
  `deadline config set/get/clear`.

### Test Strategy

Conformance tests (from `deadline-cloud-python`'s `cli_e2e/`):
- `test_farm_list_routes_through_configured_proxy` — verifies API calls
  route through the proxy when `settings.https_proxy` is set.
- `test_farm_list_without_proxy_cannot_reach_backend` — verifies that
  without proxy config, a backend reachable only through a proxy is
  inaccessible (negative control).
- `test_attachment_upload_download_routes_s3_through_configured_proxy` —
  verifies S3 transfers route through proxy with CA bundle trusted.
- `test_attachment_upload_without_proxy_cannot_reach_s3` — verifies S3
  is inaccessible without proxy/CA config (negative control).

Unit tests (Rust L1):
- Setting resolution: empty → None, whitespace → None, value → Some(trimmed).
- CA bundle `~` expansion.
- Proxy applied to HTTP connector config.
- CA bundle applied to TLS config.
- First-write-wins: second call with different values does not override.
- Warning logged when settings disagree with already-applied values.

Integration tests (Rust L2):
- CLI subprocess with `settings.https_proxy` set routes through a local
  proxy to reach the test server.
- CLI subprocess with `settings.ca_bundle` set trusts a test CA for TLS
  verification.

## Concurrency

The session cache uses a `tokio::sync::Mutex`. Public functions minimize
lock hold duration to avoid blocking concurrent callers during network I/O:

- **Cache hit:** Lock acquired briefly to clone the cached config.
- **Cache miss:** Lock acquired to check → released → config loaded
  (network I/O) → lock re-acquired to store.

This means concurrent callers on cache miss may both load the config
independently. The last writer wins, which is safe because both load
the same profile and produce identical configs.

Client construction (building `DeadlineClient`, `StsClient`) happens
entirely outside the lock. Only the raw `SdkConfig` is cached.
