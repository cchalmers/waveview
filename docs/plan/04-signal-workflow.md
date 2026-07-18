# Stage 4: Signal workflow

## Outcome

Turn loading a VCD into a practical inspect-and-arrange workflow instead of displaying every signal
in one flat list.

## Hierarchy and displayed items

- [ ] Preserve hierarchy and variable metadata when parsing rather than flattening names at load.
- [ ] Show scopes and variables in a searchable tree.
- [ ] Keep available signals separate from displayed items.
- [ ] Add one signal, a scope, or a scope recursively through commands.
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

- Loading a large hierarchy does not automatically add every signal to the canvas.
- The full add/search/group/format/reorder/remove workflow is keyboard operable.
- Reloading or switching a waveform preserves compatible displayed-item configuration.
- Scalar/vector end segments and four-state values render correctly at simulation end.
