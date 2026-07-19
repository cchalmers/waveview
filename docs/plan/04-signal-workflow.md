# Stage 4: Signal workflow

## Outcome

Turn loading a VCD into a practical inspect-and-arrange workflow with a hierarchical source browser
and a separate flat displayed-signal list.

## Hierarchy and displayed items

- [x] Preserve hierarchy and variable metadata when parsing rather than flattening names at load.
- [x] Show scopes and variables in a searchable tree.
- [x] Keep available signals separate from displayed items.
- [x] Add one signal, a scope, or a scope recursively through commands.
- [ ] Support signals, dividers, and nested groups in the displayed-item tree.
- [ ] Support fold/unfold, rename/alias, remove, reorder, and group operations.
- [ ] Retain stable displayed items when switching/reloading a compatible waveform; mark unresolved
  references and offer removal/rebinding.

## Vim workflow

- `Enter` or `a`: add the focused hierarchy item.
- `o`: open scope/group; `zc`, `zo`, `za`, `zM`, `zR` follow Vim fold conventions.
- `j`/`k` and counts navigate visible hierarchy/display rows.
- Visual-line and visual-block selections operate on multiple displayed signals.
- Operators apply to a selection: `d` removes, `y` copies, and later commands change format/color.
- `:signal add`, `:scope add`, `:group`, and related commands expose the same operations to scripts.

## Values and appearance

- [ ] Display each focused/visible signal's value at the primary cursor.
- [ ] Implement binary, hexadecimal, unsigned, signed, and ASCII vector formats.
- [ ] Give unknown and high-impedance values distinct rendering and text.
- [ ] Add per-item color, height, alias, and format state.
- [ ] Render scalar, vector, group, divider, and timeline rows through one displayed-item layout.

## Correctness and performance

- Virtualize hierarchy and displayed rows.
- Filter without rebuilding waveform data.
- Keep vertical scroll/focus stable when rows change height or groups fold.
- Ensure drag reordering and keyboard reordering produce the same command and do not alter time
  viewport state.

## Acceptance criteria

- Loading a hierarchy displays every signal by default, while the source browser makes it easy to
  remove and re-add individual signals or scopes.
- The full add/search/group/format/reorder/remove workflow is keyboard operable.
- Reloading or switching a waveform preserves compatible displayed-item configuration.
- Scalar/vector end segments and four-state values render correctly at simulation end.

## Current progress

- VCD scopes, scope kinds, variable kinds, widths, and reference indices survive conversion into
  `Waveform`; the model builds a reusable hierarchy without deriving it again from dotted labels.
- A resizable available-signal browser provides regex search, explicit scope expansion, and
  add-signal/add-scope/add-all actions. Fully added scopes disable their add button. A permanent
  activity bar selects or collapses the browser and leaves room for future waveform-source and
  server activities. The aligned signal-name panel remains the displayed-item list.
- New captures display every signal by default. Re-adds are reducer commands, deduplicate signals,
  allocate stable displayed-item IDs, and participate in display undo/redo.
- Counted `dd` removes the focused row and following rows as one undoable display change.
