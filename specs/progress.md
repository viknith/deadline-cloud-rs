# Progress

AWS Deadline Cloud Rust CLI — work item tracking.

## Getting started

Read `rust-port-workflow.md` and follow it. The Session Start section contains the
full reading checklist and gates all planning and implementation work.

## Progress

### Completed Features

All core product features are implemented and audited. 1,367+ Rust tests pass.

| Feature Area | Covers |
|-------------|--------|
| CLI commands | auth, config, farm/queue/job CRUD, bundle submit, attachments, logs, handle-web-url, MCP server |
| Job attachments | hashing, upload, download, sync-output, orchestration, S3 transfer |
| Session & auth | SSO login/logout, session caching, queue/fleet credentials, user-agent |
| Telemetry | All events (process_start, error, success/fail, sync stats, MCP latency/usage) |
| Job bundle | Parsing, validation, parameters, submission hooks, history bundles |
| Python GUI | Qt widgets/dialogs ported to `gui/`, backed by `_native.abi3.so` (PyO3) |
| DCC compatibility | Import shims for 8/9 DCCs (Blender, Maya, Nuke, C4D, VRED, 3ds Max, Unreal, After Effects) |
| Code quality | Clippy clean, ruff lint/format, behavioral parity audit passed, CLI feature parity audit passed |

### Remaining Work Items

| # | Work Item | Status | Depends On |
|---|-----------|--------|------------|
| 24 | Production distribution (maturin wheels, platform builds) | Not started | — |
| 25 | `deadline.client.api` backwards-compat shim | Not started | 24 |
| 26 | Installer pipeline update | Not started | 24 |
| 32 | Worker agent Python bindings (`job_attachments.*`) | Not started | — |
| 16f-C | DCC dependency switchover (9 repos) | Blocked | 24 |
| 16f-B | Houdini submitter rewrite | Deferred | Separate repo |
| 49 | Blender submitter UI tests in CI | Deferred | Needs purpose-built test (cross-repo PYTHONPATH conflicts) |

**#24 and #32 can proceed in parallel.**

### Python Parity Backlog (deadline-cloud-python drift since 2026-05-28)

Tracked in the table below. Updated 2026-06-29 with commits through `f2fa7b5` (v0.59.1).

**Prioritization criteria:** Only GA features that are active by default and
affect correctness/security are prioritized. Pre-launch features and opt-in
flags are deferred until they're stable and shipped.

#### Active — GA features (shipped, active by default)

| # | Feature | Python PR | Shipped in | Status |
|---|---------|-----------|-----------|--------|
| 35 | preGUI submission hook phase | #1178 | 0.57.3 | Not started |
| 43 | preGUI hooks security gate (`allow_bundle_hooks` only) | #1221 | 0.59.1 | Not started (depends on #35) |
| 40 | Consolidate auth status (remove duplicate ListFarms probe) | #1201 | 0.59.1 | Not started |
| 38 | Apply default client config to all boto clients (user-agent) | #1197 | 0.57.4 | Not started |
| 37 | Monitor session_id in telemetry | #1184 | 0.57.4 | Not started |
| 41 | AI agent invocation detection + telemetry | #1210 | 0.59.0 | Not started |
| 36 | Auto-select farm/queue when only one available | #1015 | 0.57.4 | Not started |
| 39 | Host requirements populated from job template in gui-submit | #1198 | 0.57.4 | Not started |

#### Deferred — Not GA or opt-in only

| # | Feature | Python PR | Reason |
|---|---------|-----------|--------|
| 34 | Multi-region farm support | #1202 | **Not launched.** Feature is in code (0.59.0) but not GA. Will keep changing. Port once stable. |
| 42 | Conda queue environment v2 channel migration | #1211 | **Opt-in** (`use_deadline_cloud_v2_channel=False` by default). Only matters when DCC repos consume our wheel. |
| 48 | Bulk-sync `gui/` with upstream Python UI code | — | Multi-region streaming (controller, async_runner, combo boxes), auto-select defaults, formatting drift. ~600 changed lines across 15 files. Blocked on #34 stabilizing. Do a cosmetic alignment pass independently. |

#### P2 — GUI / test infra

| # | Feature | Python PR | Status |
|---|---------|-----------|--------|
| 44 | GUI dataclasses/widgets overhaul (controllers, async runner) | Multiple | Not started |
| 45 | xa11y harness robustness (relaunch, timeout bump) | #1192, #1186 | Not started |
| 46 | Drop Python 3.8 from CI matrix | #1200 | Not started |
| 47 | xa11y bridge warm-up + cache per session | #1206 | Not started |

#### Suggested sequencing (GA items only)

1. **#35 preGUI hooks** + **#43 security gate** — correctness + security, self-contained.
2. **#40 Auth consolidation** — removes redundant ListFarms API call, simplification.
3. **#38 Default client config** + **#37 session_id telemetry** + **#41 Agent detection** — small lib changes, telemetry/user-agent correctness.
4. **#36 Auto-select farm/queue** — UX behavior, GUI + CLI.
5. **#39 Host requirements from template** — GUI bug fix.
6. **#44–47** GUI + test infra — align `gui/` + PyO3 contracts.

### Technical Debt (non-blocking)

| # | Item | Notes |
|---|------|-------|
| — | ~~Windows test twins for `#[cfg(unix)]`-gated features~~ | ✅ Done (2026-06-22). Tests made cross-platform using `cfg!(windows)` branches. |
| 33 | Error type parity audit | Python exception types flattened into generic Rust errors (`NonValidInputError` → `AssetSync`, `VFSLaunchScriptMissingError`/`VFSRunPathNotSetError` → missing, `UnsupportedProfileTypeForLoginLogout`/`PidLockAlreadyHeld`/`JobFetchFailure` → `OperationError`). Also audit openjd-rs `SnapshotError` → `JobAttachmentsError` mapping. |
| 23 | Failure case handling analysis | Systematic error handling audit across all crates |
| 22 | Fuzz testing | Robustness testing |
| 29 | `--save-debug-snapshot` bug on no-attachment bundles | Python bug to investigate |
| 15d | Audit L1 tests for conversion to L2 | Some unit tests may be better as CLI subprocess tests |
| — | KMS error guidance on download | Wrap openjd's generic S3 403 during download to detect KMS issues and suggest "ensure kms:Decrypt permission". See `download.rs:418`. |
| — | Reduce `serde_json::Value` usage (108 sites) | Incremental, address when touching those files |
| — | Stateful test server | Replace `deadline-test-server` (wiremock, static) with stateful mock supporting dynamic CRUD, call counting, configurable delays, per-test clearing. Would unify Rust L2 + xa11y GUI tests on single backend. |
| — | xa11y flakiness: STS timeout on dialog reopen | `resolve_account_id` adds 2s STS timeout when mock returns 404. Fix: add STS route to Python mock, increase dialog wait timeouts, or skip STS call when endpoint is HTTP. |
| — | JSON progress lines in `--output json` download | download-output/download-input don't emit `{"messageType":"progress",...}` lines (Python does via click progressbar callback). Low priority. |
| — | API output field ordering (HashMap iteration) | Typed SDK uses `HashMap` for maps — non-deterministic order. Audit all CLI print paths that serialize map types to ensure deterministic output. |
| — | Spec docs audit | Review `specs/` docs to ensure they reflect current implementation |
| — | Realistic test IDs | Replace hardcoded pseudo-IDs with realistic Deadline Cloud ID format |
| — | Test consolidation | Audit for redundant/overlapping tests |
| — | GUI Python code smell audit | Review ported `gui/` Python code for patterns that no longer make sense now that Rust handles business logic |
| — | Pydantic boundary validation | Explore using Pydantic to validate types crossing Rust→Python boundary (e.g. `ProgressReportMetadata`, `IniConfig`, API response dicts). Would catch contract drift. |
| — | `make test` dependency bootstrapping | `make test-bindings` requires `maturin` on PATH. Consider adding `maturin>=1.7` to `[test]` deps or documenting prerequisite. |

### Dependency Upgrades

| Crate | Current → Target | Notes |
|-------|-----------------|-------|
| rusqlite | 0.32 → 0.39 | Major, breaking changes likely |
| pyo3 | ~~0.24 → 0.28~~ | ✅ Done (upgraded to 0.29.0 in Phase 3) |
| rustls-webpki | Pinned | Advisories pinned by transitive hyper-rustls 0.24, awaiting AWS SDK upstream fix. See `deny.toml`. |

## Audit Status

- Behavioral parity: `audit_reports/archive/2026-04-17-behavioral-parity.md` — all resolved
- CLI feature parity: `audit_reports/archive/2026-05-01-cli-feature-parity.md` — all resolved
- Codebase health: `audit_reports/archive/2026-05-01-codebase-health.md` — complete

## Pending Manual Verification

**GAP-1 login retest (2026-05-12):** Re-test `./target/debug/deadline auth login`
with a fresh SSO session (no cached credentials).
