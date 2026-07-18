# Waveview implementation plan

This plan turns the current waveform prototype into a Vim-first waveform viewer while preserving
the fast native UI iteration loop. The intended product is opinionated: modal keyboard use is the
primary interface. Mouse and menu operation should remain sufficient for discovery and basic use,
but feature parity with Vim operation is not a goal.

## Architectural target

```text
keyboard / mouse / menus / Tcl
              |
              v
        ViewerCommand
              |
              v
       reducer + effects ------> file, URL, live socket, persistence
              |
              v
         ViewerState
              |
              v
     reloadable egui UI
       |             |
       v             v
    eprompt        timeline
```

The workspace should remain small:

- `waveview` is the stable application host and effect runner.
- `waveview-model` contains waveform data, stable IDs, durable viewer state, commands, and pure
  reducers. Renaming it to `waveview-core` is optional and not part of the early stages.
- `waveview-ui` contains the reloadable viewer composition and component adapters.
- `waveview-ui-reload` remains the native dynamic-library shim.
- `eprompt` is a reusable egui prompt/REPL component with no Molt dependency.
- `timeline` is a reusable timeline/view navigation component with no waveview or Rerun dependency.
- `waveserve` remains the live waveform producer.

`eprompt` and `timeline` begin as workspace members so they can evolve with waveview. Their public
APIs must use their own generic concepts rather than waveview model types, allowing either crate to
be moved to a separate repository later.

## Hot-reload rule

One reloadable dylib is enough. Component modules and reusable crates are conceptual/build
boundaries, not separate dylibs. Changes to painting, layout, prompt behavior, timeline behavior,
menus, panels, and input mapping should reload together.

Every type in the exported host/dylib function signature must be defined by the stable side and
must not change while the host is running. A common crate prevents duplicate definitions; it does
not provide a stable Rust ABI. Changing fields, enum variants, or layouts requires restarting.

The reusable widgets therefore receive borrowed frame models assembled inside `waveview-ui` and
emit actions. Their types do not appear in the exported dylib function signature. Durable data
lives in `ViewerState`; sockets, futures, dialogs, and the Molt interpreter remain host-side.

## Stages

1. [Baseline and archaeology](00-baseline-and-archaeology.md)
2. [Core state, commands, and effects](01-core-command-model.md)
3. [Reusable components and expanded hot reload](02-components-and-hot-reload.md)
4. [Vim interaction engine](03-vim-interaction.md)
5. [Signal workflow](04-signal-workflow.md)
6. [Timeline, cursor, marks, and selections](05-timeline-and-marks.md)
7. [Reusable prompt and Molt scripting](06-eprompt-and-molt.md)
8. [Formats, loading, live data, and performance](07-data-and-performance.md)
9. [Persistence, configuration, packaging, and release](08-product-hardening.md)

Each stage should end in a usable application and a clean commit series. Later stages may be
resequenced within their file, but their acceptance criteria should not be weakened silently.

## Feature references

Surfer is a requirements and design reference, especially for its command/message architecture,
hierarchy workflow, value translators, waveform switching, cursors, markers, and saved state. Its
source is EUPL-1.2, so implementation code should not be copied without an explicit licensing
decision. Reimplement the selected behavior in waveview's architecture.

The sibling `eprompt`, `timeline*`, `surfer`, and `molt` repositories are read-only archaeological
inputs until a stage explicitly imports or rewrites an idea. Their existing uncommitted work must
not be overwritten.
