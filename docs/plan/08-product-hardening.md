# Stage 8: Persistence, configuration, packaging, and release

## Outcome

Make the viewer reproducible to develop, configure, distribute, and recover after upgrades.

## Session state

- [ ] Define a versioned session format containing source identity, displayed-item tree, aliases,
  formats, colors, viewport, cursor, marks, and user-created selections worth retaining.
- [ ] Keep runtime resources and transient prompt output out of the session.
- [ ] Resolve signals by stable metadata/path and report unresolved items rather than dropping them.
- [ ] Add migrations and fixtures for every released session version.
- [ ] Support save, save-as, load, and automatic sibling/project session discovery.

## Configuration

- Vim bindings are the default and the documented primary interface.
- Allow bindings to be extended/remapped after the built-in grammar is stable; detect ambiguous or
  unreachable mappings.
- Configure theme, waveform colors, row dimensions, cursor/mark styles, snap distance, default
  formats, and startup Tcl script.
- Generate command/key reference documentation from the registries used at runtime.
- Provide a reset/default-config command and useful config parse errors.

## Undo and recovery

- Undo/redo covers display edits, grouping, reorder, aliases, formats, marks, and other user changes.
- Loading a new waveform and live data arrival are not naively stored as enormous undo snapshots.
- Autosave recovery is explicit and versioned.
- A failed UI reload, config reload, script, or session migration leaves the current viewer usable.

## Quality gates

- Reducer and Vim grammar unit tests.
- Timeline property tests.
- Parser/backend fixtures and benchmarks.
- egui snapshot tests for representative themes, values, marks, and visual modes.
- Native static, native reload, and WASM smoke tests.
- Accessibility should not regress gratuitously, but keyboard accessibility means Vim operation
  first; conventional keyboard parity is not a release blocker.

## Nix and dependency maintenance

- Pin the Rust toolchain and external sources through the flake/lockfiles.
- Make `nix flake check` the local and CI source of truth.
- Schedule dependency upgrades as explicit maintenance changes with snapshot and performance review,
  rather than continuously following latest branches.
- Package native waveview and waveserve; keep the WASM artifact reproducible.

## Release milestone

The first real release is ready when a new user can install it, load a realistic VCD, select and
arrange signals, navigate entirely with the documented Vim grammar, use marks and visual modes,
save/reload the session, execute a Tcl setup script, and connect to a live stream without consulting
the source tree.

Features such as translator plugins, transaction files, multiple viewports, framebuffer/memory
viewers, editor integration, and remote waveform filesystems remain post-1.0 candidates unless a
concrete use case promotes them.
