# Stage 2: Reusable components and expanded hot reload

## Outcome

Nearly every visible part of waveview becomes reloadable while keeping one dylib and a small
workspace. Create reusable `eprompt` and `timeline` crates without committing either to a backend
or to waveform-specific types.

## Reload boundary

Replace the exported per-row function with one stable entrypoint conceptually equivalent to:

```rust,ignore
pub fn render_viewer(ui: &mut egui::Ui, state: &mut ViewerState, out: &mut Vec<ViewerCommand>);
```

The exact signature should minimize shared types, but it may use Rust/egui references because this
is a same-toolchain development facility. Document that any shared type change requires restart.

## `waveview-ui`

Split viewer composition into modules:

- `viewer`: panel composition and focus routing.
- `wave_canvas`: waveform rows and interaction overlay.
- `signal_browser`: hierarchy/search/add UI.
- `displayed_items`: focused/selected rows, groups, and drag reordering.
- `timeline_adapter`: maps `ViewerState` to the generic timeline model/actions.
- `prompt_adapter`: maps viewer prompt state and backend results to eprompt.
- `toolbar`, `dialogs`, and `help`.

All modules compile into `waveview-ui-reload`; do not create a dylib per component.

## `eprompt` crate contract

- It depends on egui, not Molt and not waveview.
- It renders caller-owned input, history, completion candidates, output entries, and status.
- It emits generic actions such as Submit, Complete, HistoryPrevious, HistoryNext, Cancel, and
  Changed.
- Rich output is modeled as reusable text/style records, not Molt values.
- Backend evaluation is the consumer's responsibility and may be synchronous or asynchronous.
- It supports an embedded bottom bar and a standalone panel/window using the same core widget.

Create it as a fresh workspace crate. Use the sibling prototype only as a source of lessons; do not
move its dirty worktree or retain its websocket/backend coupling.

## `timeline` crate contract

- It depends on egui, not waveview and not Rerun.
- Its model is generic in meaning: integer time/sequence coordinates, visible range, optional event
  ranges, cursor/marks/selections, and a caller-provided formatter.
- It emits Pan, Zoom, Fit, SetCursor, and selection actions.
- Internal mapping uses relative `f64` screen calculations around an integer origin to preserve
  precision at high zoom.
- Gap compression is optional and disabled by default because idle waveform time is meaningful.

## Watcher work

- [x] Watch `waveview-ui`, `eprompt`, and `timeline` sources.
- [x] Rebuild only the UI dylib/shim target.
- [x] Repaint immediately after successful reload.
- [x] Keep the previous dylib active and surface a nonfatal status when compilation fails.
- [x] Detect stable-host, shared-layout, and ABI-shim edits and show a sticky restart-required
  status instead of attempting an unsafe reload.
- [x] Measure incremental builds and only consider another dylib if the normal edit cycle exceeds
  the agreed target on representative changes.

## Acceptance criteria

- Menus, panels, signal rows, timeline, prompt, and help can all change without losing a loaded
  waveform or live connection.
- `eprompt` has a backend-free demo/example.
- `timeline` has a waveform-free demo/example and mapping tests.
- Static native and WASM builds use the same component implementations without hot reload.

## Current progress

- `eprompt` is a fresh backend-free crate with caller-owned input/output, generic actions, an
  embedded widget API, and a standalone demo.
- `timeline` is a fresh waveform-free crate with relative high-precision coordinate mapping,
  cursor/selection/mark painting, generic navigation actions, mapping tests, and a standalone demo.
- Waveview's timeline now renders through the single UI dylib and maps generic timeline actions to
  `ViewerCommand`; static builds call the identical implementation directly.
- Vim command interpretation, signal-name/search painting, mode status, and generated keyboard help
  now use the same reloadable dylib. The stable host retains raw event collection and effects.
- The activity icon and available-signal header/tree are reloadable. Their host-owned panel,
  `TextEdit`, `ScrollArea`, expansion set, and waveform/display state survive library replacement.
- The waveform canvas now reloads as one component: visible row composition, hover line, persistent
  cursor, measurement drag/overlay, wheel zoom, and horizontal pan. The host retains only the
  vertical `ScrollArea`, virtualization range, and durable viewer state.
- The reload runner reports building/failure state inside the viewer and leaves the last valid dylib
  active after a compilation error. Host/shared/ABI edits produce a sticky restart-required status;
  Vim behavior and shared Vim state live in separate files so the watcher boundary is explicit.
- A representative `waveview-ui` edit rebuilt the dylib in 0.16 seconds of Cargo time (2.73 seconds
  end-to-end including `nix develop` startup), so another dylib is not justified.
