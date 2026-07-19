# Stage 3: Vim interaction engine

## Outcome

Make Vim muscle memory the primary way to operate waveview. The implementation should model Vim's
grammar rather than accumulate unrelated shortcuts.

## Input model

Implement a small, testable key interpreter outside egui painting:

```text
[count] [operator] motion
```

It tracks mode, numeric count, pending operator, partial multi-key command, last change, and jump
history. It emits `ViewerCommand`s and never directly mutates viewer state.

Initial modes:

- Normal: navigation and commands.
- Visual time (`v`): a time range on the focused signal.
- Visual line (`V`): a range of displayed rows.
- Visual block (`Ctrl-V`): a rectangle of displayed rows by time.
- Search (`/` and `?`): signal-name search initially.
- Command (`:`): the eprompt surface, backed by Molt in Stage 6.

Insert mode is not a global viewer mode. Text widgets temporarily capture input and `Esc` returns
focus to Normal mode.

## Initial motions and commands

- `j`, `k`: next/previous displayed item; counts apply.
- `gg`, `G`: first/last displayed item.
- `h`, `l`: move the time cursor left/right by a viewport-relative step.
- `b`, `w`: previous/next transition of the focused signal.
- `0`, `$`: start/end of the waveform.
- `Ctrl-F`, `Ctrl-B`: move one time viewport forward/backward.
- `Ctrl-D`, `Ctrl-U`: scroll the signal list down/up by half a page.
- `Ctrl-E`, `Ctrl-Y`: scroll the signal list down/up by one row.
- `H`, `M`, `L`: select the top, middle, or bottom visible row; counts offset from the top/bottom.
- `zt`, `zz`, `zb`: reveal focused item at top/center/bottom.
- `zi`, `zo`, `zf`: zoom in/out/fit; repeated `i` or `o` continues zooming.
- `/`, `?`, `n`, `N`, `*`: highlight search matches, move between them, and search the selected
  signal's leaf name without filtering the displayed rows.
- `dd`: remove focused displayed item.
- `J`, `K`: move focused item down/up.
- `u`, `Ctrl-R`: undo/redo display changes.
- `.`: repeat the last repeatable change.
- `y`: copy the context-appropriate name, value, time, or selected table.
- `Esc`: cancel pending grammar, selection, search/prompt focus, in that order.

Ambiguous mappings should favor a coherent Vim analogy over compatibility with other waveform
viewers. Key behavior must be documented by executable table-driven tests.

## Focus and routing

- [x] Do not run Normal-mode commands while an egui text edit owns keyboard focus.
- [ ] Give dialogs and the prompt an explicit capture layer. Dialogs are captured; the prompt is
  introduced in Stage 6.
- [x] Return to the canvas/list focus predictably on close or `Esc`.
- [x] Display the current mode, pending count/operator, and partial key sequence unobtrusively.
- [x] Generate the keyboard help window from the same binding table used for dispatch.

## Mouse and menu policy

Mouse actions should issue the same commands and provide basic loading, selection, panning, zooming,
and cursor placement. Menus should expose common actions and teach their Vim bindings. New advanced
features do not require mouse/menu parity before shipping.

## Acceptance criteria

- Key grammar tests cover counts, multi-key commands, cancellation, mode transitions, text focus,
  and repeat.
- A waveform can be navigated, zoomed, searched, rearranged, and reduced to desired signals without
  using the mouse.
- Help and status always describe the actual active keymap.

## Current progress

- The egui-independent Normal-mode interpreter handles counts and partial commands and emits only
  `ViewerCommand`s.
- Row focus, transition movement, capture bounds, viewport paging, zoom/fit, remove/reorder,
  undo/redo, and repeat are wired to the application.
- Visual modes, backward-search entry, richer context-sensitive yank, and jump history remain.
