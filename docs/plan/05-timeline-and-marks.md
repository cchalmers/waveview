# Stage 5: Timeline, cursor, marks, and selections

## Outcome

Finish the generic `timeline` crate and give waveview Vim-style positions, marks, jumps, and visual
selections rather than adopting Surfer's numbered-cursor model.

## Generic timeline work

Port, with attribution and license review where necessary, the useful ideas from the old
Rerun-derived experiment rather than depending on it:

- [ ] Integer-origin plus relative `f64` time/screen mapping.
- [ ] Round-trip mapping property tests at large timestamps and extreme zoom.
- [ ] Pan and zoom-around-pointer operations.
- [ ] Adaptive major/minor tick spacing and caller-provided formatting.
- [ ] Visible-range culling.
- [ ] Cursor, mark, and selection painting supplied as generic overlay records.
- [ ] Optional discontinuous range compression, off by default.

Do not port `re_log_types`, playback state, or Rerun panel composition. The reusable crate should be
useful for traces, media, logs, and waveform viewers without knowing any of those domains.

## Waveview position model

A Vim position is `(DisplayedItemId, time)`. A linewise position may ignore the exact time when
appropriate, but it should still retain it for returning to the previous location.

Marks follow Vim conventions:

- `m{a-z}` sets a local mark at the focused signal and cursor time.
- `` `{mark} `` jumps to its exact signal and time.
- `'{mark}` jumps to the marked signal while preserving/revealing time according to linewise Vim
  semantics.
- `` `` `` and `''` return to the previous jump position.
- `:marks` lists marks; `:delmarks` removes them.
- Uppercase/global marks are deferred until multiple files/sessions have a stable identity model.

If a marked signal disappears on reload, keep the mark unresolved with its stored signal path and
offer rebinding when a matching signal returns.

## Cursor and transition navigation

- Primary cursor placement snaps to a nearby transition on the focused signal.
- `h`/`l` and counts move across transitions while keeping the cursor visible.
- Jumping, search results, start/end motions, and marks update a Vim-style jump list.
- `Ctrl-O`/`Ctrl-I` traverse older/newer jump-list positions.
- A measurement readout shows cursor time and selection/mark deltas.

## Visual modes

- `v`: time interval on the focused signal; `h`/`l`, `0`/`$`, marks, and searches extend it.
- `V`: contiguous displayed-item range.
- `Ctrl-V`: rectangular signal-by-time selection.
- `o` moves the active end of a visual selection.
- `Esc` returns to Normal mode without moving the cursor.
- `y` copies a stable textual representation; operators may consume selections in later stages.

## Acceptance criteria

- `timeline` builds and demonstrates independently of waveview.
- Marks, exact/line jumps, jump history, and all three visual modes survive UI hot reload.
- Marks and selections persist in session state and degrade safely after waveform replacement.
- Timeline mapping tests cover high timestamp precision and zoom limits.

## Current progress

- Local `a-z` marks store a displayed-item ID and exact capture time. `m{a-z}`, exact `` `{mark} ``,
  linewise `'{mark}`, and previous-jump `` `` ``/`''` syntax run through the Vim command engine.
- Exact jumps restore signal and time; linewise jumps restore the signal while preserving the current
  time. Both reveal the destination row, and previous-jump positions toggle in Vim fashion.
- Mark times are painted across the timeline and waveform canvas. `marks`, `delmarks <names>`, and
  `delmarks all` provide the initial Molt lifecycle and report unresolved removed signals safely.
- Marks currently reset on waveform replacement; compatible-source preservation and unresolved-path
  rebinding remain part of the session/source identity work.
