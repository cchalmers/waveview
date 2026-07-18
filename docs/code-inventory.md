# Code archaeology inventory

This inventory records the disposition of prototype code before deletion or architectural work.
Git history remains the archive; useful behavior should become a requirement or test rather than a
large commented implementation.

## Active product behavior

| Area | Current location | Disposition |
| --- | --- | --- |
| File, URL, drop, and live loading | `src/app.rs`, `src/live.rs` | Keep; move behind host effects in Stage 1. |
| Search/filter | `src/app.rs` | Keep; replace flat filtering with hierarchy/display search. |
| Synchronized labels and waveform rows | `src/app.rs` | Keep behavior; replace coupled scroll geometry with viewport/displayed-item state. |
| Drag reorder | `src/app.rs` | Keep; express as the same command used by Vim reorder. |
| Cursor and drag measurement | `src/app.rs` | Keep; replace hover-only geometry with cursor, marks, and visual selections. |
| Virtualized visible rows | `src/app.rs` | Keep and test with variable row heights/groups. |
| Compact four-state signals | `waveview-model/src/vcd.rs` | Keep until the backend evaluation in Stage 7. |

## Requirements extracted from TODOs

| Requirement | Source | Planned stage |
| --- | --- | --- |
| Preserve vertical location when row height changes | `src/app.rs` | Stage 1/4 viewport and row model. |
| Reordering must not alter horizontal scroll | `src/app.rs` | Stage 1 command/viewport split. |
| Render unknown and high-impedance values distinctly | `waveview-ui/src/wave.rs` | Stage 0 fixture, Stage 4 rendering. |
| Extend the last value to capture end | `waveview-ui/src/wave.rs` | Stage 0 parser test, Stage 4 rendering test. |
| Support formats other than debug output | `waveview-ui/src/wave.rs` | Stage 4 value formatting. |
| Replace initial-fit and scroll correction hacks | `src/app.rs` comments | Stage 1 viewport model and Stage 5 timeline. |

## Unreachable and prototype source

| Source | Finding | Disposition |
| --- | --- | --- |
| `src/list.rs` | Uncompiled variable-height list prototype. Its cumulative-height mutation has edge cases, but the lower-bound idea is useful. | Do not revive directly. Cover variable-height row lookup in the Stage 4 displayed-item model, then delete. |
| `src/my_scroll.rs` | Uncompiled fork of an old egui `ScrollArea`, approximately 850 lines. | Do not maintain a framework fork. Port only a demonstrated missing behavior; otherwise delete after the viewport split. |
| `src_old/` | Untracked pre-revival backup. | Leave untouched until the owner decides whether to archive/delete it. Git history already contains the tracked evolution. |
| `Cargo.lock_`, `flake-old.nix` | Untracked backups. | Leave untouched; not inputs to the current build. |
| `timeline` branch and sibling `timeline*` | Rerun-derived timeline mapping, ticks, and early `j`/`k` selection. | Reimplement selected algorithms in the generic `timeline` crate with tests and license review. |
| Sibling `eprompt` | Coupled Molt/websocket/egui prototype in a dirty worktree. | Use as design input only; create a backend-free workspace crate. |

## Comment cleanup policy

- Remove template tutorial comments and `if false` example windows when their surrounding area is
  next edited.
- Replace debug `eprintln!` calls with purposeful logging or remove them.
- Keep comments that explain VCD semantics, safety constraints, or non-obvious precision choices.
- Convert actionable TODOs into the staged plan or an issue before removing the comment.
- Never delete untracked backups as part of routine cleanup.
