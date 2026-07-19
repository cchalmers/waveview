# Molt dependency audit

Waveview uses Molt as a host-side Tcl interpreter. `eprompt` deliberately has no Molt dependency.

The sibling checkout at `../molt` was inspected on 2026-07-19 and is not used as a path
dependency. Its `master` branch contains two local commits above upstream commit
`f7c601008a32b42934e790d132b6b2ac134e61da`:

- `faf7b10` is primarily a historical `cargo clippy --fix` pass.
- `af0ddce` exposes Molt's internal `commands` module and makes small macro/style changes. The old
  eprompt prototype needed this because it manually registered each standard command.

The sibling also has uncommitted dependency/style edits and a deleted workspace manifest. None are
required by Waveview. Molt's public `Interp::new` already installs the standard command set, while
`save_context` and `add_context_command` allow Waveview to replace `puts` with transcript capture.

Molt is unmaintained and Waveview is expected to need interpreter features of its own. The minimal
interpreter crate from the sibling fork is therefore vendored as the local `molt` path dependency. It
keeps the BSD-3-Clause license and explicit upstream/fork attribution while omitting the book,
shell, application, benchmarks, test harness, and repository-level tooling. The imported code is
based on upstream commit `f7c6010`, the fork commits through `af0ddce`, and the fork's current
compatibility edits as of the audit date.

Waveview depends only on this local crate; it does not depend on the dirty sibling path or a
network Git checkout. Future changes should stay focused on the embedded interpreter and retain
their provenance. Waveview-specific commands remain in the Waveview host rather than Molt itself.

The crate is excluded from first-party workspace Clippy selection because the inherited 2020 code
has a substantially older lint baseline. Its unit suite is run explicitly with
`cargo test --manifest-path molt/Cargo.toml`; Waveview's prompt integration remains covered by the
normal workspace checks.
