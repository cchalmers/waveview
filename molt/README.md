# Molt for Waveview

This is the minimal interpreter crate from Christopher Chalmers' Molt fork, vendored so Waveview
can support and extend its Tcl interface on native and WebAssembly targets. It intentionally omits
the upstream book, shell, application, benchmarks, test harness, and repository-level tooling.

It is an in-repository path dependency but is excluded from Waveview's first-party Cargo workspace
lint selection. The inherited 2020 code has its own lint baseline; run its tests explicitly with
`cargo test --manifest-path molt/Cargo.toml`.

Molt was created by William H. Duquette and contributors and is distributed under the BSD
3-Clause license in [`LICENSE.txt`](LICENSE.txt). The imported code is based on upstream commit
`f7c601008a32b42934e790d132b6b2ac134e61da`, plus the fork's Clippy/WASM compatibility work through
`af0ddcea1143eda30c5cd00a8092f441343d1c48` and its current working-tree compatibility fixes as of
2026-07-19.

Keep this crate focused on Waveview's embedded interpreter. Generic fixes should remain clearly
attributed; Waveview commands belong in the Waveview host rather than this crate.
