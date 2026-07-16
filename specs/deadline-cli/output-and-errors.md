# Output Formatting and Error Suggestions

How the CLI formats output and helps users recover from errors.

## Output Format Auto-Detection

Commands that accept `--output` (see table below) auto-detect the
format when the flag is omitted:

| stdout is... | Resolved format | Rationale |
|-------------|----------------|-----------|
| Interactive TTY | `verbose` | Human at a terminal wants readable output |
| Not a TTY (pipe, redirect, closed) | `json` | Scripts, CI, and agents want parseable output |

An explicit `--output verbose` or `--output json` always wins regardless
of TTY state.

### Commands with `--output`

| Command | Accepts `--output` |
|---------|-------------------|
| `auth status` | ✓ |
| `config show` | ✓ |
| `bundle gui-submit` | ✓ |
| `job download-output` | ✓ |
| `job download-input` | ✓ |
| `job wait` | ✓ |
| `job logs` | ✓ |

Commands not in this table are unaffected — they produce a fixed format.

### Edge Cases

- **Broken stdout** (stream closed, `isatty()` errors): treated as
  non-TTY → json.
- **Case sensitivity**: `--output` values are case-insensitive
  (`JSON`, `Json`, `json` all resolve to json).
- **Subprocess tests**: CLI invoked as a subprocess (no TTY) defaults
  to json. Tests asserting verbose output must pass `--output verbose`
  explicitly.

### Help Text

The `--output` help text documents the auto-detection:

> Specifies the output format of the messages printed to stdout.
> VERBOSE: Displays messages in a human-readable text format.
> JSON: Displays messages in JSON line format, so that the info can be
> easily parsed/consumed by custom scripts.
> When this option is not specified, the format is chosen automatically:
> VERBOSE when stdout is an interactive terminal, and JSON otherwise
> (for example when the output is piped, redirected, or run without a
> TTY such as in CI or by an agent).

---

## Resource Suggestions on Error

When an API call fails with `AccessDeniedException`, `ResourceNotFoundException`,
or `ValidationException`, the CLI attempts to list alternative resources and
appends suggestions to the error message. This helps users who mistyped an ID
or lack permissions to a specific resource.

### Trigger Conditions

`suggest_resources_on_client_error()` checks the error message string for the
three exception names. Other error types (throttling, internal server error)
do not trigger suggestions.

### Dispatch by Operation Name

The suggestion chain is determined by which API operation failed, not by
which resource IDs happen to be available. Each caller passes the operation
name (e.g. `"GetQueue"`, `"GetFleet"`) and the function dispatches to the
correct suggestion chain. This matches Python's `_OPERATION_GROUPS` pattern.

| Operation group | Operations | Suggestion chain |
|----------------|------------|-----------------|
| queue | `GetQueue`, `ListQueues`, `ListQueueEnvironments` | queues → farms |
| farm | `GetFarm`, `ListFarms` | farms |
| fleet | `GetFleet`, `ListFleets` | fleets → farms |
| worker | `GetWorker`, `SearchWorkers` | workers → fleets |
| job | `GetJob`, `ListJobs`, `SearchJobs`, `CreateJob` | jobs → queues → farms |
| storage_profile | `GetStorageProfileForQueue`, `ListStorageProfilesForQueue` | (not yet implemented) |

Unknown operation names fall back to listing farms.

Each step in the chain calls the corresponding List API. If the list call
succeeds and returns results, the suggestions are formatted and returned.
If it fails (e.g., the user also lacks List permissions), the chain
continues to the next level.

If all list calls fail: returns a message suggesting the IAM policy may be
missing List permissions.

### Output Format

```
Available queues in farm farm-abc123:
  queue-def456  My Render Queue
  queue-ghi789  My Test Queue
```

Shows up to 10 items with ID and display name. If more exist, appends
"... and N more".

### Known Difference from Python

The error message prefix differs. Python's boto3 `ClientError` formats as:
```
An error occurred (AccessDeniedException) when calling the GetQueue operation: <message>
```
Rust's `format_sdk_error` produces:
```
AccessDeniedException: <message>
```
The suggestion content (resource list) is identical. The error prefix
difference is inherent to how the AWS SDK for Rust formats errors vs
boto3 and applies to all API error messages, not just suggestions.
