# Development workflow

Nix pins the Rust toolchain and native/WASM build tools. Run commands from the repository root.

## Native application

```sh
nix develop
cargo run -- vcds/edge_cases.vcd
```

This is the normal static debug build. A specific capture may be passed after `--`; files can also
be opened, dropped onto the window, or fetched by URL.

## Native UI hot reload

```sh
nix run .#reload -- vcds/edge_cases.vcd
```

Within `nix develop`, `reload-waveview vcds/edge_cases.vcd` is equivalent. The command builds and
watches the UI dynamic library, then runs the stable host with the `reload` feature. A successful
library replacement requests an egui repaint and retains host-owned state.

The reload boundary is deliberately based on files rather than guesses about the meaning of a Rust
edit:

| Change | Running viewer |
| --- | --- |
| `waveview-ui/**` | Hot reloaded |
| `eprompt/**` | Hot reloaded |
| `timeline/**` | Hot reloaded |
| `waveview-model/src/vim.rs` | Hot reloaded |
| `src/**` (the stable application host) | Restart required |
| `waveview-model/**`, except `src/vim.rs` | Restart required |
| `waveview-ui-reload/**` (the dynamic-library ABI shim) | Restart required |
| Workspace `Cargo.toml`, `Cargo.lock`, or `flake.nix` | Restart required |
| Documentation, plans, VCD fixtures, and unrelated workspace crates | No effect on the running viewer |

In practical terms, waveform/timeline painting, canvas hover/cursor/measurement interaction,
wheel zoom and horizontal pan, activity icons, the available-signal browser header/tree,
signal-name/search painting, Vim command interpretation, mode status, and keyboard help can be
changed without losing the loaded capture or viewer state. `eprompt` is rebuilt into the same
library, although Waveview does not use it yet. Application/panel orchestration, stateful egui
containers (`TextEdit`, `ScrollArea`, drag-and-drop), file loading, shared
model/search/waveform types and behavior, Vim state fields, and exported dynamic-library signatures
need a restart. `vim_types.rs` is intentionally separate from `vim.rs` so that this distinction is
unambiguous.

The hot-side code is split by editing surface while still producing one dylib:

| Component | Source |
| --- | --- |
| Activity-bar controls | `waveview-ui/src/activity.rs` |
| Available-signal hierarchy | `waveview-ui/src/signal_browser.rs` |
| Menu contents and controls | `waveview-ui/src/menus.rs` |
| Displayed signal rows/search highlighting | `waveview-ui/src/displayed_items.rs` |
| Waveform rows and canvas interaction | `waveview-ui/src/canvas.rs`, `waveview-ui/src/wave.rs` |
| Timeline-to-viewer action mapping | `waveview-ui/src/timeline_adapter.rs` |
| Vim status and key help | `waveview-ui/src/vim_ui.rs` |

`waveview-ui/src/lib.rs` is the narrow public facade used by both static and reload builds. Keep
durable application state out of these component modules.

The host deliberately owns stateful egui containers and virtualization offsets, then passes their
inner `Ui` plus borrowed application state to reloadable render functions. This keeps egui state
created by one dylib from surviving after that dylib is unloaded while allowing most visible
contents and interactions to reload. Top-level menu popups follow the same rule: the host creates
the popup, reloadable code paints its contents and emits a `MenuAction`, and the host performs the
file, window, or model effect.

A failed hot build leaves the previous UI active and shows a nonfatal failure message; compiler
diagnostics remain in the terminal. Editing a restart-only file shows a persistent yellow `restart
required; host or shared code changed` message in the viewer's bottom status bar. The warning is
intentionally sticky: subsequent successful UI reloads do not clear it, because the running host is
still stale.

Reload mode disables egui's selectable-label feature. egui 0.35 stores label selection as a plugin
keyed by Rust `TypeId`, and those IDs change when the dylib is replaced. Leaving it enabled can make
the first label painted by the new library panic. Waveview's explicit copy commands are unaffected,
and normal static builds retain selectable labels.

## WASM application

```sh
nix develop .#wasm
trunk serve --open
```

Hot library reload is native-only. WASM uses the same UI implementation, statically linked.

## Verification

The complete reproducible check is:

```sh
nix flake check
```

It builds native and WASM applications, runs the workspace tests, checks formatting, and runs
native, reload-feature, and WASM Clippy checks with warnings denied. Faster checks while editing are:

```sh
nix develop -c cargo test --workspace
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo fmt --all --check
nix develop .#wasm -c trunk build
```

## Stage 0 timing baseline

On 2026-07-18, the first observed `cargo test --workspace` after entering the Nix environment took
15.3 seconds of Cargo build/test time and 21 seconds wall time with some dependencies already cached.
The UI dynamic-library target took 15.3 seconds in a colder build, 0.36 seconds for a warmed
no-change build, and 3.65 seconds after touching one `waveview-ui` source file. These are local
reference points rather than controlled benchmarks; repeat them after introducing the component
crates and investigate if a typical one-file reload becomes materially slower.

On 2026-07-19, after adding `eprompt`, `timeline`, and the expanded Vim/search UI boundary, a
representative `waveview-ui` edit rebuilt `waveview-ui-reload` in 0.16 seconds of Cargo time and
2.73 seconds wall time including `nix develop` startup. One reloadable dylib remains comfortably
fast enough.
