# Handoff

Current in-flight work. Read this at the start of every session before
consulting the Work Items table in `specs/progress.md`.

Active work item: **#35 Phase 2+3 — Submit Dialog (Rust QML) + PyO3 show_submit_dialog()**

---

## #35 Phase 2+3 — Submit Dialog + DCC Integration

**Baseline:** 1,493 Rust tests, 32 PyO3 binding tests (all green)

### Current State

Batches 2a and 2b are complete. Batch 2c is **in progress** — Rust backend
is done, QML frontend needs to be written.

The submit dialog currently has:
- 4 tabs (Shared settings, Job-specific, Attachments, Host requirements)
- Submit/Export/Cancel buttons working end-to-end
- Farm/queue/storage profile cascading selection
- Queue parameter dynamic form (TextField/ComboBox/SpinBox/CheckBox)
- Attachment lists (auto-detected + user-added, add/remove)
- Progress dialog with hashing/upload bars, cancel, result JSON
- Auth status bar with login/logout

**Tab 3 (Host requirements) is a placeholder** — only two radio buttons,
no actual form fields. Help button is a no-op.

### Batch 2c — What's Done (Rust backend)

All Rust code is implemented and tested:

1. **`host_requirements_model.rs`** (new file) — cxx-qt bridge model with:
   - 16 properties (use_custom_requirements, OS bools, hardware i32s, custom strings)
   - `serialize()` → returns JSON string or empty
   - `add/remove/update_custom_amount` and `add/remove/update_custom_attribute`
   - Registered in `build.rs` and `lib.rs`

2. **`logic/host_requirements.rs`** — added:
   - `HostRequirementsModelState` struct
   - `serialize_from_model_state()` — converts UI state to JSON (GiB→MiB)
   - `validate_custom_name()` — OpenJD naming regex
   - Pipe/semicolon parsing helpers
   - 5 new L1 tests (all passing)

3. **`submit_model.rs`** — added:
   - `host_requirements_json` field
   - `set_host_requirements_json()` invokable
   - HR wiring into both `submit()` and `export_bundle()`

4. **xa11y tests** (5 new in `test_bundle_gui_submit_controls.py`):
   - `test_custom_requirements_disabled_by_default`
   - `test_os_checkboxes_present_when_custom_selected`
   - `test_hardware_spinboxes_present_when_custom_selected`
   - `test_host_requirements_in_exported_bundle`
   - `test_help_button_opens_dialog`
   - These tests currently FAIL (expected — QML not written yet)

### Batch 2c — What's Left (QML frontend)

The QML in `SubmitDialog.qml` needs these changes:

1. **Add `HostRequirementsModel { id: hostRequirementsModel }`** to model instances

2. **Add `Accessible.onPressAction: tabBar.currentIndex = N`** to each TabButton
   (required for xa11y tab switching on macOS)

3. **Replace Tab 3 placeholder** with full form:
   - Radio buttons bound to `hostRequirementsModel.use_custom_requirements`
   - OS/CPU section: untitled GroupBox with two rows ("Operating system" + checkboxes,
     "CPU architecture" + checkboxes)
   - Hardware section: GroupBox "Hardware requirements" with GridLayout, SpinBoxes
     using `textFromValue: function(value) { return value < 0 ? "" : value.toString() }`
   - Custom section: GroupBox "Custom host requirements" with:
     - "ℹ More info" label (light blue ℹ, toggles tooltip)
     - Repeater for amounts (name + min/max + delete button)
     - Repeater for attributes (name + Any/All radio + values list + delete)
     - "Add amount" / "Add attribute" buttons
   - **CRITICAL**: Use a separate QML component file for the custom requirements
     section. Nested Repeaters on a single line cause `index` shadowing bugs
     (inner delegate's index resolves to outer Repeater's index). Write
     `HostRequirementsTab.qml` or at minimum use multi-line delegate format.

4. **Attachment tab labels**: "General submission settings" box around checkbox,
   "Attach input files", "Attach input directories", "Specify output directories"

5. **Help dialog**: QML `Dialog` with title "About Deadline Cloud Submitter",
   version TextArea, Copy/Close buttons. Wire Help button: `onClicked: helpDialog.open()`.
   Do NOT put `Accessible.name` on the Dialog (it's not an Item).

6. **Wire HR into submit/export buttons** in QML:
   ```qml
   submitModel.set_host_requirements_json(hostRequirementsModel.serialize())
   ```

7. **Pre-existing issues to be aware of (NOT in Batch 2c scope)**:
   - Binding loop in parameter Repeater (delegate reads `parameters_json` which
     it also writes to via `onTextChanged`). Fix: use `Component.onCompleted`
     for initial value, not reactive binding.
   - Bundle parameters (e.g. `Message` in simple_job) don't load because
     `parameterModel` only fetches queue env params from API. Need
     `load_bundle_parameters()` that reads template's `parameterDefinitions`.
   - Parameters show on Tab 0 (Shared settings) but should be on Tab 1
     (Job-specific settings) per Python UI.

### Lessons Learned (QML pitfalls)

- **No semicolons between child items**: `RowLayout { Label {} ; TextField {} }` is INVALID.
  Use `RowLayout { Label {} TextField {} }` or multi-line format.
- **Nested Repeater `index` shadowing**: Inner Repeater's `index` is shadowed by outer
  Repeater's `index` when both are on the same line. Use separate component files
  or `property int valIdx: index` with care.
- **`Accessible.name` on Dialog**: QML `Dialog` derives from Popup, not Item.
  `Accessible` can only attach to Item-derived types.
- **StackLayout + accessibility**: On macOS, only the current tab's content is
  exposed to the accessibility tree. `Accessible.onPressAction` on TabButtons
  is required for xa11y `press()` to actually switch tabs.

### Status: Step 3 in progress. Rust backend complete, QML frontend pending.

---

## Completed items

- **#35 Batch 2b — Queue Parameters + Attachments UI (2026-05-25)** —
  ParameterListModel, AttachmentModel, QML dynamic parameter form,
  attachment lists, ComboBox accessibility fix, farm/queue race fix,
  xa11y test infrastructure fix, mock backend queue environment support.
  1,483→1,488 Rust tests. 35→38 xa11y tests.

- **#35 Batch 2a — Submit Action + Progress + Export (2026-05-24–25)** —
  SubmitModel, ProgressModel, SubmitDialog.qml, ProgressDialog.qml,
  logic/submit.rs, CLI gui-submit wiring, job history bundles, --output json,
  dark mode, cancel handling, tilde expansion. 1,421→1,483 tests.

- **#35 Batch 1 — Config/Auth/Resource Models (2026-05-21–22)** —
  ConfigModel, AuthModel, ResourceModel, ConfigDialog.qml, logic.rs,
  CLI config gui wiring, xa11y test infrastructure. 1,355→1,421 tests.

- **#34 — Library/CLI boundary refactor (2026-05-20)** — Removed `&IniConfig`
  from all library signatures. 1,355 Rust tests, 373 Python tests pass.

- **Audit findings — Batches A-J (2026-05-15–16)** — 45 findings resolved.

- **#31 — Crate Restructure (2026-05-14)** — All 12 steps done.

---

## Pending Manual Verification

**GAP-1 login retest (2026-05-12):** Re-test `./target/debug/deadline auth login`
with a fresh SSO session (no cached credentials).

---

## #16f — DCC Submitter Dependency Switchover

**Status:** Batches A1-A3 ✅ Done. Batch B deferred (Houdini, separate repo).
Batch C blocked on #24 (production distribution).
