# Handoff

Current in-flight work. Read this at the start of every session before
consulting the Work Items table in `specs/progress.md`.

## Active: Cross-repo UI conformance — PR #6 (nearly green)

**Status:** PR branch `ci/cross-repo-ui-tests` — fix pushed, awaiting CI confirmation.
Bug fix for failing JSON output test committed + Blender UI tests added.

**Branch:** `ci/cross-repo-ui-tests`

**What the PR does:** Deletes duplicated `pytests/ui_accessibility/` tests and
replaces them with a workflow that clones `deadline-cloud-python`, installs our
Rust-backed package (`pip install -e ".[gui]"` + `cargo build -p deadline-cli`),
and runs their `test/ui/` suite against our binary via `DEADLINE_BINARY`.

### Fix applied (2026-06-30)

**`test_json_output_contains_submitted_status_and_job_id`** — was failing on all 3 OS:
```
AssertionError: No JSON object found in stdout: ''
```

**Root cause:** `_gui_entry.py` set `submitter._close_event_receiver = submitter.close`
in JSON mode. This closed the submitter dialog but left the progress dialog open
(it's a child widget). With the progress dialog still showing, `QApplication.exec()`
never returned. The Python subprocess hung, the Rust binary hung, and the test's
fallback SIGTERM killed both processes before stdout was written.

**Fix:** Removed the `_close_event_receiver` override. The Python CLI never
auto-closes — the test (or caller) dismisses the dialog via accessibility, which
closes the submitter + child progress dialog, allows `exec()` to return, and the
JSON is printed normally.

**Linux-only flake:** `TestOutputJsonCancel::test_json_output_reports_canceled` —
same root cause. Should be fixed by the same change.

### Blender submitter UI tests added (2026-06-30)

Added `blender-ui-linux`, `blender-ui-macos`, `blender-ui-windows` jobs to
`python.yml`. Each installs Blender, builds our Rust binary, installs our
deadline package + the Blender addon, and runs the Python repo's
`test/blender_submitter_ui/` suite. Windows uses `continue-on-error: true`.

### Upstream PR pending

`fix/mock-server-localhost` branch on `deadline-cloud-python` — changes mock
server to bind `localhost` instead of `127.0.0.1`. Once merged, the `sed` patch
step in our workflow can be removed. File the PR when ready.

---

## Previous: CI/CD Hardening — ✅ Complete

---

### Phase 1: Conformance workflow verification ✅ MERGED

Completed in PR #2. Nightly conformance workflow dispatched manually
post-merge — confirmed passing.

### Phase 2: Verify conformance workflow passes ✅ DONE

Conformance run passed (2026-06-19, 8m8s). Added `pull_request` trigger
with path filters (`crates/`, `conformance/`, `Cargo.toml`, `Cargo.lock`)
so conformance runs on code-touching PRs in addition to the nightly schedule.

### Phase 3: pyo3 0.24 → 0.29 upgrade ✅ DONE

Upgraded pyo3 and pythonize to 0.29.0. Migration changes:
- `Python::with_gil` → `Python::attach`, `allow_threads` → `detach`
- `PyObject` → `Py<PyAny>`, `FromPyObject` → dual-lifetime + `FromPyObjectOwned`
- Removed `unsafe impl Send` (Py<PyAny> is Send); kept `unsafe impl Sync`
- Removed RUSTSEC-2026-0176/0177 from deny.toml

### Phase 4: Unpin rust-toolchain.toml ✅ DONE

Removed `rust-toolchain.toml` (was 1.94.0). CI now uses latest stable.
Fixed 8 new clippy lints from Rust 1.96: `map_unwrap_or`, `is_ok_and`,
`checked_div`, `sort_by_key(Reverse)`, trailing comma, `from_mins`.

### Phase 5: Python test workflows ✅ DONE

Added two workflows:

**`.github/workflows/python.yml`** (PR-triggered, path-filtered):
- Bindings tests (`pytests/bindings/`) on all 3 OSes
- xa11y GUI accessibility tests (`pytests/ui_accessibility/`) on all 3 OSes
- Linux: Xvfb + dbus + AT-SPI2 (mirrors deadline-cloud-python's ui_tests.yml)
- macOS: TCC accessibility permission grant
- Windows: UIA works out of the box

**`.github/workflows/gui-drift.yml`** (nightly):
- Clones deadline-cloud-python, extracts `test/ui/` test function names
- Compares against our `pytests/ui_accessibility/` test function names
- Fails if Python repo has tests we haven't ported (surfaces new GUI tests)

### Phase 6: Windows test twins ✅ DONE

Made 35 unix-only tests cross-platform using `cfg!(windows)` branches
for platform-appropriate commands (.cmd vs .sh, Windows symlink APIs).
No production code changes. Branch: `ci/windows-test-twins`.

Files updated:
- `crates/deadline-lib/src/bundle/hooks.rs` (10 unit tests)
- `crates/deadline-lib/tests/bundle/hooks.rs` (6 integration tests + TestHandler)
- `crates/deadline-cli/tests/cli/auth.rs` (6 tests + helpers)
- `crates/deadline-cli/tests/cli/bundle_hooks.rs` (8 tests + helpers)
- `crates/deadline-cli/tests/cli/bundle.rs` (1 symlink test)
- `crates/deadline-lib/tests/bundle/loader.rs` (3 symlink tests)
- `crates/deadline-cli/tests/cli/queue_sync_output.rs` (1 read-only path test)

---

## GitHub CI/CD Architecture

### Inner loop: `.github/workflows/ci.yml`

Triggered on every push to `mainline` and every PR targeting
`mainline`, `release`, `patch_*`, `feature_*`.

| Job | Runs on | What it does |
|-----|---------|-------------|
| **Rustfmt** | ubuntu | `cargo fmt --all -- --check` |
| **cargo-deny** | ubuntu | License, advisory, ban, source checks |
| **Build & Test** | ubuntu, macOS, Windows (matrix) | Build all targets, clippy `-D warnings`, `cargo test --workspace`, doctests |
| **Documentation** | ubuntu | `cargo doc --no-deps --workspace` with `-D warnings` |

Key infrastructure:
- No toolchain pin — CI uses latest stable (removed in PR #3)
- `*.localhost` host entries on macOS/Windows (AWS SDK `management.` prefix)
- Cargo cache keyed on `(os, rustc-hash, Cargo.lock-hash)` with stale eviction
- `concurrency` cancels in-progress PR runs; never cancels mainline
- `fail-fast: false` — all 3 OSes complete even if one fails

### Outer loop: `.github/workflows/conformance.yml`

Nightly (07:00 UTC) + PR (path-filtered) + manual dispatch. Replays
`deadline-cloud-python`'s `test/cli_e2e/` against the Rust binary. Pytest
plugin rewrites localhost URLs for the SDK's host prefix. xfail allowlist
for known gaps.

### Python tests: `.github/workflows/python.yml`

PR-triggered (path-filtered: `crates/deadline-python-bindings/`, `gui/`,
`pytests/`, `pyproject.toml`). Runs:
- **Bindings** (`pytests/bindings/`) — tests `deadline._native` PyO3 module, all 3 OSes
- **GUI xa11y** (`pytests/ui_accessibility/`) — accessibility-driven tests of
  the real Qt GUI, all 3 OSes. Linux needs Xvfb + AT-SPI; macOS needs TCC
  grant; Windows UIA works natively.

Our `pytests/ui_accessibility/` is a superset of `deadline-cloud-python`'s
`test/ui/` — both test the same GUI code but ours uses `_gui_entry.py`
directly (no CLI indirection) and includes additional coverage.

### GUI drift detection: `.github/workflows/gui-drift.yml`

Nightly (07:30 UTC). Clones `deadline-cloud-python`, compares test function
names in their `test/ui/` against our `pytests/ui_accessibility/`. Fails if
they've added tests we haven't ported. Does not execute tests — just a
parity tracker.

---

## Deferred: Rust GUI Rewrite (QML)

**Status:** Deferred indefinitely. Work preserved on branch `qml-gui-wip`.
The Python Qt GUI in `gui/` remains the production GUI.

---

## Completed items (recent)

- **CI/CD Hardening phase 6 (2026-06-22)** — Windows test twins. 35 unix-only
  tests made cross-platform. All CI/CD hardening phases (1–6) complete.

- **CI/CD Hardening phases 2–4 (2026-06-19)** — PR #3. Conformance PR trigger,
  pyo3 0.24→0.29 (removed 2 security advisory ignores), unpinned toolchain
  (Rust 1.96, 8 clippy lints fixed). All 7 CI checks green on 3 OSes.

- **Cross-OS CI — GitHub Actions (2026-06-18)** — PR #2. Full 3-OS gate.
  Fixed 5 real Windows bugs. 1,141 tests passing on Windows.

- **Python repo parity — telemetry + UI (2026-05-28)** — process_start,
  record_error_with_trace, HoverRadioButton. Merged (a5d99cf, ce941a8).

---

## Pending Manual Verification

**GAP-1 login retest (2026-05-12):** Re-test `./target/debug/deadline auth login`
with a fresh SSO session (no cached credentials).

---

## #16f — DCC Submitter Dependency Switchover

**Status:** Batches A1-A3 ✅ Done. Batch B deferred (Houdini, separate repo).
Batch C blocked on #24 (production distribution).
