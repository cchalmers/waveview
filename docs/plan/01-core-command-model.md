# Stage 1: Core state, commands, and effects

## Outcome

Replace direct mutation throughout `TemplateApp::ui` with a durable model and a single command
path. Vim, menus, mouse gestures, Tcl, and future remote control must be able to request identical
operations.

## State split

Introduce these concepts in `waveview-model`:

- `SignalId` and `DisplayedItemId`: stable identity independent of row position.
- `Waveform`: loaded hierarchy, signal metadata, transitions, timescale, and end time.
- `DisplayedItem`: signal, group, divider, or timeline row.
- `Viewport`: visible time origin/span, with pure pan, zoom, fit, and reveal operations.
- `CursorState`: primary time cursor and focused displayed item.
- `SelectionState`: selected items and visual selection description.
- `ViewerState`: durable/persistable composition of the above plus display preferences.
- `UiSessionState`: prompt/search text, current Vim mode, pending count/operator, and temporary
  dialogs that should survive a UI library reload.

Host-only `Runtime` state stays in `waveview`:

- File dialog futures and downloads.
- Live socket and incremental receive buffers.
- Error reporting channels.
- Molt interpreter, once introduced.
- Reload observer and repaint plumbing.

## Command path

- [x] Define the initial `ViewerCommand` vocabulary for capture, viewport, cursor, and effect
  operations.
- [x] Define the initial `EffectRequest` vocabulary for file and network operations.
- [x] Implement a pure reducer returning zero or more effects.
- [x] Make file, URL, dropped-file, and live-load completion feed commands back into the same
  reducer.
- [x] Convert Reset, zoom, search, reorder, cursor measurement, and loaded-wave replacement first.
- [x] Convert remaining domain-level menu and mouse mutations; purely presentational panel/dialog
  state remains host-local.
- [x] Add command-level tests for viewport, cursor/measurement, waveform replacement, stable
  display identity, invalid ordering/focus, and display undo/redo without constructing an egui
  context.

Adding a `ViewerCommand` variant changes a type shared across the reload boundary and therefore
requires a host restart. That is acceptable: hot reload optimizes UI implementation, not schema
development.

## Invariants

- Row indices are derived and never used as durable identities.
- Removing or reloading a signal cannot leave dangling focus, mark, or selection references.
- Viewport operations do not depend on egui scroll offsets.
- Effects do not mutate `ViewerState` behind the reducer.
- Commands are serializable where practical, so session files and scripting can share vocabulary.

## Replacement policy

- Opening a file or URL, dropping a file, and Reset fit the new capture and clear search/cursor
  session state.
- A subsequent live update preserves and clamps the current viewport; the first live snapshot fits
  the capture.
- Replacing a capture rebuilds displayed items with fresh capture-local stable IDs and clears the
  display undo history, so focus cannot refer to the previous capture.

## Acceptance criteria

- `TemplateApp` is primarily `ViewerState + Runtime`, not a collection of UI geometry fields.
- Menus and mouse input enqueue commands rather than directly changing domain state.
- Reducer tests cover invalid IDs, clamping, waveform replacement, undoable display changes, and
  viewport operations.
- Loading and live updates still preserve or reset view state according to an explicit policy.
