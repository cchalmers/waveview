# Stage 0: Baseline and archaeology

## Outcome

Establish a reliable baseline and convert old experiments into explicit requirements, tests, or
deletions. This stage should not substantially redesign the UI.

## Work

- [x] Record the current native, reload, and WASM startup commands in one developer document.
- [x] Add a small checked-in VCD fixture covering scalar, vector, unknown, high-impedance, repeated
  values, and a final value extending to the end time.
- [x] Make `nix flake check` run formatting, tests, Clippy, and the appropriate build smoke tests.
- [x] Record approximate clean and incremental reload build times. Use these as the baseline for
  later crate decisions.
- [x] Inventory every TODO and commented block in `src/app.rs`, `waveview-ui`, and
  `waveview-model` as one of: active requirement, useful experiment, debugging residue, or dead
  template code.
- [x] Review `src/list.rs`, `src/my_scroll.rs`, `src_old`, the `timeline` branch, and sibling
  `timeline-workspace` for behavior that is not represented elsewhere.
- [ ] Preserve useful behavior as a test or a short design note, then remove unreachable source and
  obsolete commented implementations.
- [ ] Keep the current user edits and unrelated untracked backup files untouched.

## Requirements recovered so far

- Unknown and high-impedance values must be visually distinct.
- The final signal segment must extend to the simulation end.
- Changing row height must preserve the apparent vertical location.
- Signal drag-and-drop must not perturb horizontal waveform scroll.
- Initial fit and zoom-around-pointer need a real viewport model rather than scroll-area correction
  hacks.
- Search, cursor measurement, URL/file/drop loading, and live VCD are product features, not demos.
- The `j`/`k` experiment in the old timeline work is the seed of the modal navigation design.

## Acceptance criteria

- `nix flake check` is the documented one-command verification path.
- The fixture and parser/rendering unit tests cover the recovered correctness requirements.
- No compiled-out module or large commented implementation remains without an explicit reason.
- Native static, native reload, and WASM builds still start with the same user-visible behavior.

## Non-goals

- New navigation behavior.
- A new waveform backend.
- Moving UI code across the reload boundary.
