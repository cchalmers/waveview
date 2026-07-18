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

Changes to Rust types shared by the host and dynamic library are not reload-safe even if the
watcher rebuilds successfully. Restart after changing fields, variants, layouts, or exported
function signatures.

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
