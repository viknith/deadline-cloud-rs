# Integration Plan: Rust into deadline-cloud-python

**Updated:** 2026-06-29  
**Status:** Ready to execute

---

## Goal

Ship `deadline-cloud-python` with Rust business logic compiled into the
same `deadline` wheel via PyO3. Python becomes thin wrappers + GUI.
Other engineers keep working in the Python repo normally.

---

## Decisions (2026-06-29)

- **No PoC needed.** The pattern is proven — openjd-model-for-python shipped
  v0.10.0 with Rust bindings on June 23. Same team, same infra.
- **Publish `deadline-lib` to crates.io first.** This is the single blocker.
  A path-dep PoC doesn't de-risk anything the openjd team hasn't already proven.
- **Use `_deadline_rs` as module name** (matches `_openjd_rs` convention).
- **In-place replacement with feature flag**, not versioned `_v1` API. We
  control both sides. `DEADLINE_USE_NATIVE=0` env var disables Rust fallback
  during transition.
- **Copy `_build_backend.py` from openjd-model-for-python.** It's generic
  (maturin + setuptools_scm VCS versioning). Minor adaptation needed.
- **Repo structure:** `viknith/deadline-cloud-rs` (detached, independent) and
  `viknith/deadline-cloud` (fork of aws-deadline/deadline-cloud) for dev work.

---

## How openjd Did It (the proven pattern)

```
openjd-model-for-python/
├── rust-bindings/Cargo.toml    ← depends on openjd-{expr,model,sessions} from crates.io
├── rust-bindings/src/          ← PyO3 #[pyfunction]/#[pyclass] wrappers
├── _build_backend.py           ← PEP 517 wrapper: maturin + setuptools_scm version injection
├── pyproject.toml              ← build-backend = "_build_backend", [tool.maturin] config
└── src/openjd/
    ├── _openjd_rs.abi3.so      ← single binary for py3.9–3.14+
    └── model/
        ├── __init__.py         ← old API (pydantic, still works)
        └── _v1/__init__.py     ← new API: `from openjd._openjd_rs import ...`
```

Key details:
- `abi3-py39` — one wheel per platform, works across Python versions
- `[patch.crates-io]` block (commented out) for local iteration
- `_build_backend.py` patches `pyproject.toml` at build time to inject VCS version
- Old code keeps working; new Rust path is opt-in via `_v1` namespace
- 6-platform wheel matrix in CI (Linux x86/arm, macOS x86/arm, Windows x86/arm)
- Tested in production SMF on Linux and Windows before merging

Current state: openjd-model v0.10.1 is on PyPI with Rust. The worker agent
pins `openjd-model < 0.10` to exclude it until validated. Sessions bindings
exist on a branch but aren't merged yet.

---

## Architecture (Target)

```
deadline-cloud-python/                  ← Ships to PyPI as "deadline"
├── rust-bindings/
│   ├── Cargo.toml                      ← depends on deadline-lib from crates.io
│   └── src/lib.rs                      ← #[pymodule] deadline._deadline_rs
├── _build_backend.py                   ← copied from openjd-model-for-python
├── pyproject.toml                      ← build-backend = "_build_backend"
└── src/deadline/
    ├── _deadline_rs.abi3.so            ← compiled Rust
    └── client/
        ├── config/config_file.py       ← delegates to _deadline_rs with fallback
        ├── api/                        ← delegates to _deadline_rs
        └── ui/                         ← stays Python (PySide6)
```

---

## Step 1: Publish `deadline-lib` to crates.io

| Task | Status |
|------|--------|
| Detach `viknith/deadline-cloud-rs` from fork network | Done |
| Add `repository`, `keywords`, `categories` to Cargo.toml | Not started |
| Add `release-plz.toml` (copy from openjd-rs) | Not started |
| Add `.github/workflows/release-plz.yml` | Not started |
| `cargo publish -p deadline-lib --dry-run` | Not started |
| First `cargo publish -p deadline-lib` (manual) | Not started |
| Set up crates.io Trusted Publishing (OIDC) | Not started |

---

## Step 2: Add `rust-bindings/` to deadline-cloud-python

```toml
# rust-bindings/Cargo.toml
[package]
name = "deadline-python"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
name = "_deadline_rs"
crate-type = ["cdylib"]

[dependencies]
deadline-lib = "0.1.0"
pyo3 = { version = "0.29", features = ["abi3-py39"] }

[features]
extension-module = ["pyo3/extension-module"]

# Local dev override (uncomment when iterating on deadline-lib locally)
# [patch.crates-io]
# deadline-lib = { path = "../../deadline-cloud-rs/crates/deadline-lib" }
```

Expose the same functions already working in `deadline-cloud-rs/crates/deadline-python-bindings/src/`:
- Config: `get_setting`, `set_setting`, `read_config`
- Auth: `check_auth_status`, `login`, `logout`
- Resources: `list_farms`, `get_farm`, `list_queues`, `get_queue`, etc.
- Submission: `create_job_from_job_bundle`
- Telemetry: `TelemetryClient`

---

## Step 3: Switch build system

1. Copy `_build_backend.py` from openjd-model-for-python
2. Update `pyproject.toml`:
   ```toml
   [build-system]
   requires = ["maturin>=1.0,<2.0", "setuptools_scm"]
   build-backend = "_build_backend"
   backend-path = ["."]

   [tool.maturin]
   python-source = "src"
   module-name = "deadline._deadline_rs"
   manifest-path = "rust-bindings/Cargo.toml"
   features = ["extension-module"]
   ```
3. Add Rust toolchain to CI, add `rust_quality.yml` (clippy, cargo test)
4. Use `reusable_maturin_prerelease.yml` for 6-platform wheel matrix

---

## Step 4: Delegate Python functions to Rust (incremental)

Pattern for each function:

```python
import os
_USE_NATIVE = os.environ.get("DEADLINE_USE_NATIVE", "1") != "0"

try:
    if _USE_NATIVE:
        from deadline._deadline_rs import get_setting as _rs_get_setting
    else:
        raise ImportError("disabled")
except ImportError:
    _rs_get_setting = None

def get_setting(setting_name, config=None):
    if _rs_get_setting is not None:
        return _rs_get_setting(setting_name, config_path=str(get_config_file_path()))
    # existing Python implementation below
    ...
```

Order of migration (low risk → high impact):
1. **Config** — `get_setting`, `set_setting` (pure file I/O, easy to validate)
2. **Auth + Telemetry** — removes duplicate ListFarms call, fire-and-forget telemetry
3. **Resource listing** — farm/queue/job APIs (after this, new features land Rust-first)
4. **Job submission** — `create_job_from_job_bundle` (removes boto3 from critical path)
5. **Job attachments** — hashing, upload, download (biggest perf win, parallel Rust I/O)

---

## Step 5: CI/CD and PR workflow

- Existing Python tests must pass with `DEADLINE_USE_NATIVE=1` (default)
- Add CI job that runs tests with `DEADLINE_USE_NATIVE=0` (fallback path)
- PR template with cross-port checklist:
  ```markdown
  ### Cross-port to deadline-cloud-rs
  - [ ] No runtime behavior change, **or**
  - [ ] Matching change in deadline-cloud-rs: *<link>*, **or**
  - [ ] Tracking issue filed: *<link>*
  ```

---

## Intersection with Worker Agent

The openjd team has a `bindings-rs` branch on `deadline-cloud-worker-agent`
that uses openjd Rust bindings for sessions. Our work item #32 (worker agent
bindings for `job_attachments.*`) would give the worker agent Rust-backed
attachments too. Both converge on the same goal: worker agent running
primarily on Rust.

The openjd team uses a runtime toggle (v0 vs v1 API selection) so the
worker agent can be deployed to production AMIs and switched via userdata.
We should coordinate on this mechanism.

---

## References

- openjd-model-for-python `_build_backend.py`: [GitHub](https://github.com/OpenJobDescription/openjd-model-for-python/blob/mainline/_build_backend.py)
- openjd-model-for-python `rust-bindings/`: [GitHub](https://github.com/OpenJobDescription/openjd-model-for-python/tree/mainline/rust-bindings)
- openjd-rs release-plz config: `openjd-rs/release-plz.toml`
- OpenJD EXPR Quip (context on strategy): https://quip-amazon.com/1TVEAPjihSKm
- deadline-cloud-rs PyO3 bindings: `crates/deadline-python-bindings/src/lib.rs`
- Worker agent Rust bindings branch: `mwiebe/deadline-cloud-worker-agent@bindings-rs`
