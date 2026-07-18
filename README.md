# waveview

An egui waveform viewer for Value Change Dump (VCD) files. It runs as a native application and in
the browser, and includes a small server for following a VCD while a simulation is running.

## Development

Nix supplies the pinned Rust toolchain and all build tools:

```sh
nix develop
cargo run -- vcds/example.vcd
```

Files can also be opened from the menu, dropped onto the window, or fetched by URL. For the web
build:

```sh
nix develop .#wasm
trunk serve --open
```

The useful verification commands are:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
trunk build
nix flake check
```

See the [development workflow](docs/development.md) for native, hot-reload, and WASM commands and
the [implementation plan](docs/plan/README.md) for the staged Vim-first viewer work.

## Native UI hot reload

The native host owns loaded VCD data, viewport state, file operations, and live WebSocket
connections. Waveform painting lives in a small dynamic library, so it can be rebuilt and replaced
without restarting the host or losing that state.

Run the viewer and UI watcher together with:

```sh
nix run .#reload -- path/to/capture.vcd
```

Alternatively, from `nix develop`, run `reload-waveview`. Changes under `waveview-ui` rebuild the
native-only `waveview-ui-reload` shim and request an immediate egui repaint. Changes to
`waveview-model` also rebuild the dylib, but changing the layout of shared model types while the host
is running is unsafe; restart after those changes. Normal `cargo run`, release, and WASM builds
remain statically linked.

## Live VCD server

Start a simulator writing a VCD, then run:

```sh
cargo run -p waveserve -- --input path/to/live.vcd
```

The health endpoint is `http://127.0.0.1:9123/health` and the WebSocket endpoint is
`ws://127.0.0.1:9123/ws`. In waveview, choose **File → Connect live…** and connect to that URL.
The same client works in native and WASM builds. See [the protocol description](docs/live-protocol.md).

## Repository layout

- `waveview-model`: stable VCD parsing and indexed signal storage shared across the reload boundary.
- `waveview-ui`: reloadable waveform painting.
- `waveview-ui-reload`: native-only dynamic-library shim used by the reload feature.
- `src/app.rs`: viewer state and UI.
- `waveserve`: live VCD WebSocket server.
- `assets` and `index.html`: Trunk web application assets.
- `docs`: design notes and the older generated GitHub Pages build.

The `timeline` branch and sibling `timeline-workspace` directory contain an experimental
Rerun-derived timeline and dynamic-library hot reload work. They are intentionally kept separate
until the core viewer and live transport are stable.
