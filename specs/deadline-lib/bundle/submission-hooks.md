# Submission Hooks

External scripts that run during job submission. Three phases exist:

| Phase | When it runs | Failure behavior |
|-------|-------------|-----------------|
| Pre-GUI | Before the gui-submit dialog opens | Blocks the dialog (error) |
| Pre-submission | Before attachment hashing/upload | Cancels submission (error) |
| Post-submission | After successful CreateJob | Warning only, never blocks |

## Hook Sources

Two independent sources, each gated by its own config setting:

| Source | Config Setting | Default |
|--------|---------------|---------|
| Bundle (`hooks.yaml` in job bundle dir) | `settings.allow_bundle_hooks` | `false` |
| Environment (`DEADLINE_HOOKS_DIR` env var) | `settings.allow_environment_hooks` | `false` |

When both are active, environment hooks run first, then bundle hooks.
This ordering applies to all three phases.

### Multi-source execution

Each source is a separate execution context with its own script-resolve
directory. Hooks from different sources are NOT merged into a single
flat list — they execute as independent groups so that relative script
paths resolve against the correct directory for their source.

For pre- and post-submission hooks on the CLI `bundle submit` path:

1. Collect sources: environment first (if valid and enabled), then
   bundle (if hooks exist and enabled).
2. Show a single confirmation prompt listing all sources' hooks with
   per-source labeling (see [Confirmation Prompt](#confirmation-prompt)).
3. Execute pre-submission hooks by iterating sources in order. The
   merged output from the earlier source is passed as input to the next,
   so a later source can override an earlier source's changes.
4. Execute post-submission hooks by iterating sources in order. No
   payload threading — post-hooks are fire-and-forget.

For pre-GUI hooks, the same multi-source pattern applies: each source's
hooks run independently, and their output is accumulated with later
sources overriding earlier ones for scalars and per-key merging for
parameters.

### Source deduplication

When `DEADLINE_HOOKS_DIR` resolves (via realpath) to the same directory
as the job bundle, the two sources are effectively the same `hooks.yaml`.
In this case the directory is treated as a single source — hooks are not
loaded and run twice. The deduplicated source is permitted if *either*
`allow_bundle_hooks` or `allow_environment_hooks` is enabled.

### Source labels

Each source carries a label identifying its origin:

| Source | Label |
|--------|-------|
| Bundle | `"job bundle"` |
| Environment | `"environment (DEADLINE_HOOKS_DIR)"` |

These labels appear in the confirmation prompt header and in any
"hooks present but disabled" guidance messages.

## Hook Configuration Format

`hooks.yaml` or `hooks.json` in the bundle or env hooks directory:

```yaml
version: "1.0"
preGUI:
  - command: python3
    args: [prefill.py]
    timeout: 10
preSubmission:
  - command: python3
    args: [validate.py]
    timeout: 30
    env:
      VALIDATION_LEVEL: strict
postSubmission:
  - command: python3
    args: [notify.py]
```

All three phase keys (`preGUI`, `preSubmission`, `postSubmission`) are
optional. Each is a list of hook definitions.

Hook definition fields: `command` (required string), `args` (optional
list, default `[]`), `timeout` (optional positive integer, default `60`),
`env` (optional map, default `{}`).

## Execution Model

Hooks are subprocesses. Each receives:
- **stdin**: `HookMetadata` as JSON (job name, farm/queue IDs, parameters,
  asset references, submission payload)
- **env vars**: `DEADLINE_JOB_NAME`, `DEADLINE_PRIORITY`, `DEADLINE_FARM_ID`,
  `DEADLINE_QUEUE_ID`, `DEADLINE_JOB_BUNDLE_DIR`, plus optional
  `DEADLINE_STORAGE_PROFILE_ID` and `DEADLINE_JOB_ID` (post-hooks only)
- **custom env**: from the hook's `env` field

### Pre-GUI hooks

Run sequentially before the gui-submit dialog opens. The dialog does not
appear until all pre-GUI hooks complete successfully.

- Non-zero exit or timeout → error, dialog does not open.
- Invalid JSON on stdout → error, dialog does not open.
- Empty stdout → no changes to dialog defaults.
- Valid JSON on stdout → merged into the dialog's initial field values
  (see [Pre-GUI Hook Output](#pre-gui-hook-output)).

### Pre-submission hooks

Run sequentially before attachment hashing/upload.

- Non-zero exit or timeout → submission canceled (error).
- Invalid JSON on stdout → submission canceled (error).
- Empty stdout → payload unchanged.
- Valid JSON on stdout → merged into the submission
  (see [Pre-Submission Hook Output](#pre-submission-hook-output)).

### Post-submission hooks

Run sequentially after successful CreateJob.

- Failures only log warnings, never block.
- `job_id` available in metadata.

### Stderr streaming (progress output)

Hook stderr is streamed to the user line-by-line as it arrives while the
hook runs. This provides live feedback for slow hooks (e.g., generating
auth tokens) that would otherwise appear to hang.

Behavioral rules:

1. Each stderr line emitted by the hook is forwarded to the user
   immediately (not buffered until exit), prefixed with the hook's
   phase and 1-based index. Example:
   ```
     [pre-submission hook 1] Authenticating with service A...
     [pre-submission hook 1] Done.
   ```

2. Stdout is NOT streamed — it is captured whole after the hook exits.
   Stdout is reserved for the JSON contract (hook output payload).

3. On hook failure, the failure report shows stdout (if non-empty) but
   does NOT repeat stderr. Stderr was already shown live, so repeating
   it would be redundant.

4. After execution completes (success, failure, or timeout), no further
   stderr lines are delivered to the user. A lingering child process
   that inherited the pipe file descriptors and continues writing stderr
   must not produce visible output past the point of completion.

5. The grace period for draining output after the hook process exits
   (or is killed) is 5 seconds. If output is still arriving after the
   grace period (due to a lingering child holding the pipe open), the
   hook is treated as timed out.

6. Post-completion stderr is retained only at debug log level. It is
   not shown to the user again.

**Scope boundaries:**

- **In scope:** All three hook phases (pre-GUI, pre-submission,
  post-submission) stream stderr identically.
- **Out of scope:** stdin is written in one shot and closed; there is no
  interactive stdin/stdout dialog with the hook subprocess.

## Path Resolution

1. Absolute paths → used as-is
2. Relative paths → resolved from script resolve dir
3. Command names → searched in system PATH

The script resolve dir is normally the bundle directory, but
`.hooks_origin` file (written by GUI for job history bundles) redirects
resolution to the original bundle location.

## Confirmation Prompt

When hooks are present and `auto_accept` is false, a confirmation
message lists all hooks grouped by source. Each source block identifies
its origin and directory.

Without an interactive callback (or user declining), submission is
canceled with "Job submission canceled (user declined hooks)."

The confirmation is rendered once across all sources (not per-source).

Per-source block format:

```
This {source label} contains submission hooks that will execute on your machine:

  Pre-GUI hooks:
    [1] python3 prefill.py

  Pre-submission hooks:
    [1] python3 validate.py

  Post-submission hooks:
    [1] python3 notify.py

  Location: /path/to/source/dir

```

When multiple sources are active, their blocks are concatenated.
The final line of the full message is "Do you want to run these hooks?"

Example with both sources:
```
This environment (DEADLINE_HOOKS_DIR) contains submission hooks that will execute on your machine:

  Pre-submission hooks:
    [1] python3 env_validate.py

  Location: /etc/deadline/hooks

This job bundle contains submission hooks that will execute on your machine:

  Pre-submission hooks:
    [1] python3 validate.py

  Post-submission hooks:
    [1] python3 notify.py

  Location: /path/to/bundle

Do you want to run these hooks?
```

---

## Pre-GUI Hook Output

Pre-GUI hooks output JSON to pre-populate the submission dialog.

### Allowed output fields

| Field | Type | Effect |
|-------|------|--------|
| `name` | string | Pre-fills the job name field |
| `description` | string | Pre-fills the job description field |
| `parameters` | object | Pre-fills parameter values (see below) |

Any other top-level key is rejected with an error naming the
unrecognised fields and listing the allowed ones.

### Parameters map

The `parameters` object maps parameter names to values:

- **Job template parameters** use their name directly (e.g., `"SceneFile"`).
- **Shared job properties** use the `deadline:` prefix:
  - `deadline:priority` (integer, 0–100)
  - `deadline:maxFailedTasksCount` (integer)
  - `deadline:maxRetriesPerTask` (integer)
  - `deadline:maxWorkerCount` (integer)
  - `deadline:targetTaskRunStatus` (string: `"READY"` or `"SUSPENDED"`)

### Merge behavior across multiple hooks

When multiple pre-GUI hooks run (e.g., one from environment, one from
bundle):

- `name` and `description`: later hooks overwrite earlier values.
- `parameters`: merged per-key. A later hook overrides the same key but
  does not discard keys set by an earlier hook.

### Precedence

CLI-supplied `--parameter` values always take precedence over
hook-supplied values. A hook cannot override a parameter that was
explicitly passed on the command line.

### Security gate

Pre-GUI hooks from the job bundle are gated **exclusively** on
`settings.allow_bundle_hooks`. They do not run if only
`settings.allow_environment_hooks` is enabled.

Pre-GUI hooks from `DEADLINE_HOOKS_DIR` are gated on
`settings.allow_environment_hooks`.

When hooks are present but the relevant setting is disabled, a warning
is logged:
- Bundle: "Note: Job bundle contains preGUI hooks but bundle hooks are
  disabled.\nEnable with: deadline config set settings.allow_bundle_hooks true"
- Environment: "Note: DEADLINE_HOOKS_DIR contains preGUI hooks but
  environment hooks are disabled.\nEnable with: deadline config set
  settings.allow_environment_hooks true"

### Metadata passed to pre-GUI hooks

Pre-GUI hooks receive `job_bundle_dir` set to the **job bundle being
submitted** (not the environment hooks directory). This matches the
pre/post-submission contract. Relative hook script paths still resolve
against the hook's own source directory.

---

## Pre-GUI Hooks API (DCC Entry Point)

Pre-GUI hooks are available to DCC submitters (Maya, Nuke, Blender,
etc.) via a headless entry point that does not require Qt. This allows
in-application submitters to run pre-GUI hooks before building their
submission dialog.

### Inputs

The caller provides:

- **Job bundle directory** — the bundle being submitted. DCC submitters
  have no on-disk bundle at pre-GUI time and omit this, so their only
  hook source is `DEADLINE_HOOKS_DIR`.
- **Job name** — initial job name passed to hooks.
- **Parameters** — current parameter values (name → value map).
- **Submitter name** — identity string (e.g., "maya", "nuke",
  "JobBundle").
- **Priority** — initial priority (default 50).
- **Farm ID, Queue ID, Storage Profile ID** — when not provided, fall
  back to the corresponding config defaults.

### Confirmation

The caller provides a confirmation callback:

- When omitted: hooks run without prompting (for DCC submitters that
  always want hooks, or when `auto_accept` is true).
- When provided: called once with the list of hook sources. If the user
  declines, submission is canceled with "Job submission canceled (user
  declined hooks)."

The standard GUI uses a Qt dialog as the confirmation callback, but
the pre-GUI hooks entry point itself is Qt-free.

### Behavioral rules

1. When no bundle directory is provided, only the `DEADLINE_HOOKS_DIR`
   source is consulted. The bundle source is skipped entirely.

2. When the bundle directory is empty, a `hooks.yaml` or `hooks.json`
   in the process's current working directory is NOT loaded. This
   prevents accidental code execution for DCC submitters that have no
   bundle.

3. Source selection uses the same gating and deduplication logic as the
   submission path (see [Hook Sources](#hook-sources)).

4. The returned merged output contains any of `name`, `description`,
   `parameters`. An empty result is returned if no pre-GUI hooks ran.

5. Later sources (bundle) override earlier sources (environment) for
   scalar fields (`name`, `description`). Parameters merge per-key
   (later overrides same key, preserves keys from earlier).

### Applying pre-GUI output to settings

The merged pre-GUI output is applied to the submitter's settings:

1. `name` → sets the initial job name.
2. `description` → sets the initial job description.
3. For each parameter in the output:
   - If the submitter has a template-parameter list and the name
     matches: update that parameter's value in place.
   - Otherwise: place the parameter in the shared values (queue
     parameters, `deadline:` job properties, etc.).
4. CLI-supplied parameter names always take precedence over hook values.

When the submitter has NO template-parameter list (DCC submitters),
every hook parameter flows to the shared values.

### Scope boundaries

- **In scope:** Headless pre-GUI hook execution for DCC and standalone
  GUI submitters. Source selection, confirmation injection, output
  merging, and generic output application.
- **Out of scope:** The Qt confirmation dialog (that is a UI concern).
  DCC-specific settings types (each DCC defines its own).

---

## Pre-Submission Hook Output

Pre-submission hooks output JSON to modify the submission. The following
top-level fields are recognized:

### `attachments.assetReferences`

Merges asset references into the upload set (existing behavior). Each
nested key (`inputFilenames`, `inputDirectories`, `outputDirectories`,
`referencedPaths`) replaces the corresponding key in the original.
Keys not present in the hook output are preserved.

### `priority`

Overrides the job's priority. The hook-supplied integer replaces
whatever priority was configured (CLI `--priority`, config default, or
the hardcoded default of 50).

### `template`

Replaces the job template content. The hook-supplied string becomes the
template sent to CreateJob.

If no `template` key is present in hook output, the template file is
re-read from disk after hooks complete — this allows a hook to modify
the template file on disk without needing to emit the full content on
stdout.

### `parameters`

A map of parameter name → value. Allows hooks to change job parameter
values that reach CreateJob.

When multiple hooks run, `parameters` maps are merged per-key (later
hooks override the same key but preserve keys from earlier hooks). This
matches the merge behavior of `attachments.assetReferences`.

**Two channels for parameter changes:**

A pre-submission hook can change parameters in two ways, both honored:

1. **On-disk rewrite:** The hook modifies `parameter_values.yaml` or
   `parameter_values.json` in the job bundle directory. After hooks
   complete, the file is re-read and parameters are re-resolved.

2. **Stdout `parameters` map:** The hook emits a `parameters` key in
   its JSON output. These values are applied on top of any disk changes.

In both cases, CLI `--parameter` values retain final precedence — a hook
cannot override a parameter the user explicitly supplied on the command
line.

**PATH parameter constraint:** A PATH parameter emitted on stdout must
be an absolute path. A relative PATH value on stdout is rejected with an
error:

> Pre-submission hook emitted a relative PATH value for parameter
> 'NAME': 'VALUE'. Hooks must emit absolute paths for PATH parameters
> on stdout, since a hook does not run from the submitting working
> directory. Use an absolute path (e.g. join with DEADLINE_JOB_BUNDLE_DIR)
> or rewrite parameter_values.yaml on disk.

(On-disk PATH values in `parameter_values.yaml` are resolved against the
job bundle directory, following the existing resolution rules.)

**Asset reference recomputation:** When parameters are re-resolved after
a hook changes them, input/output directory references are recomputed
from scratch. This prevents a stale directory from being uploaded when a
hook redirects a PATH parameter to a new location. Only the new location
is uploaded.

**Known-path recomputation:** After re-resolution, the "known asset
paths" list (used to detect unexpected paths during attachment scanning)
is rebuilt from the re-resolved parameters. This ensures a hook that
redirects a PATH parameter doesn't trigger the "unknown path" warning or
cancel submission for the new location.

---

## Payload Merging (updated)

Hook output is shallow-merged into the submission payload with two
special cases:

1. **`attachments.assetReferences`**: each nested key is replaced
   independently. Other attachment fields are preserved.
2. **`parameters`**: merged per-key. Later hooks override the same key
   but preserve keys set by earlier hooks.

All other top-level keys are overwritten by the hook output.

---

## Where Each Phase Runs

| Submission method | Pre-GUI | Pre-submission | Post-submission |
|-------------------|---------|----------------|-----------------|
| `deadline bundle submit` (CLI) | ✗ | ✓ | ✓ |
| `deadline bundle gui-submit` (standalone GUI) | ✓ | ✓ | ✓ |
| DCC submitters (Maya, Blender, etc.) | ✓ (env source only when no bundle dir) | ✓ | ✓ |

---

## Test Strategy

### Stderr streaming

- Execute a hook that writes multiple stderr lines with delays. Assert
  each line arrives with the correct prefix before the hook exits.
  Assert stdout JSON is not streamed.
- Execute a hook that fails. Assert the failure report includes stdout
  but not stderr.
- Lingering child test: hook spawns a detached child holding the pipe,
  then exits. Assert execution completes within grace period and no
  output arrives after completion.
- CLI-level: `bundle submit` with a hook that writes stderr progress.
  Assert prefix-formatted lines appear in CLI output.

### Multi-source execution

- Environment and bundle sources both have pre-submission hooks. Assert
  both run in order, with payload threaded (source 2 sees source 1's
  output). Assert script resolution uses each source's own directory.
- Confirmation message with two sources includes both source labels and
  "Location:" lines.
- CLI-level: `bundle submit` with `DEADLINE_HOOKS_DIR` set and bundle
  hooks. Assert both hooks execute (sentinel files from each).

### Pre-GUI API (DCC entry point)

- With no bundle directory, only the env source is loaded. Bundle
  hooks.yaml is not consulted.
- Empty bundle directory does not load hooks from the current working
  directory.
- Output application routes template params to the parameter list and
  non-template params to shared values. CLI-supplied names are not
  overridden.
- Confirmation callback returning false raises cancellation.

---

## Differences from Python

| Aspect | Python | Rust |
|--------|--------|------|
| Hook execution messages | Via configurable callback | Via the submission handler interface |
| Streamed stderr delivery | Same callback | Same submission handler interface |
| Confirmation source path label | `"Location:"` | `"Location:"` (matches) |
| Confirmation bundle path | `script_resolve_dir` (from `.hooks_origin`) | `job_bundle_dir` |
| Pre-GUI hooks for DCC | Python module importable by DCC plugins | Exposed via PyO3 bindings |

The confirmation bundle path difference only matters for the GUI job
history case. The pre-GUI hooks are implemented in Rust and exposed to
Python DCC plugins via the PyO3 bindings module.
