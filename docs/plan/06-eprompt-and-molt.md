# Stage 6: Reusable prompt and Molt scripting

## Outcome

Use the reusable `eprompt` widget for Vim command/search surfaces and embed Molt in waveview as one
backend. `eprompt` itself remains unaware of Tcl, Molt, waveforms, and websockets.

## Finish `eprompt`

- [ ] Support editable input, scrollback, history, completion candidates, selection, status, and
  error/result styling.
- [ ] Define a small action API rather than accepting an evaluator callback inside egui painting.
- [ ] Allow consumers to supply completion candidates incrementally.
- [ ] Support command (`:`), forward search (`/`), and backward search (`?`) presentation modes.
- [ ] Provide configurable prompt text and key handling while preserving expected Vim prompt keys.
- [ ] Add a standalone example with a trivial calculator/echo backend and no Molt dependency.
- [ ] Test history navigation, completion selection, submit/cancel, multiline output, and bounded
  scrollback.

## Molt host integration

Add a waveview host module, not a dependency from `eprompt`:

- Own `molt::Interp` in stable runtime state.
- Register commands through contexts that enqueue `ViewerCommand` or `EffectRequest` values.
- Capture `puts` and evaluation results as generic prompt output entries.
- Give query commands a read-only snapshot rather than mutable access to `ViewerState`.
- Evaluate submitted scripts outside the UI borrow to avoid `RefCell` re-entrancy.
- Request repaint when asynchronous command results arrive.

Before integration, audit the dirty sibling Molt fork, record why it differs from upstream, and pin
the chosen source/revision. Do not silently depend on an uncommitted path checkout.

See [`../molt-audit.md`](../molt-audit.md) for the completed audit and pinned upstream revision.

## Initial Tcl vocabulary

```tcl
open file.vcd
reload
scope add top.cpu
scope add -recursive top.cpu
signal add top.cpu.pc
signal remove
signal format hex
signal focus next
zoom fit
zoom to 100ns 250ns
cursor set 123ns
transition next
mark set a
mark goto a
source setup.tcl
connect ws://127.0.0.1:9123/ws
```

Command names and arguments should be regular Tcl words/options, not a second parser embedded in a
single command string. Tcl commands translate to the same viewer commands used by Vim and menus.

## Script lifecycle

- Optional startup script from the user configuration directory.
- Explicit `source` for project/session scripts.
- Command history persisted separately from viewer state.
- Errors include Tcl stack/error information in scrollback without crashing the viewer.
- Long-running or network work returns immediately and completes through host effects.

## Acceptance criteria

- `eprompt` can be reused by a non-waveview egui application without Molt.
- `:` commands can reproduce core loading, display, navigation, formatting, and mark operations.
- A Tcl script can configure a freshly loaded waveform deterministically.
- Invalid scripts, backend errors, and asynchronous failures remain visible and nonfatal.

## Current progress

- `:` now opens a Waveview command console with host-owned focus, input, persistent history,
  bounded transcript, and a persistent Molt interpreter.
- Standard Tcl evaluation works; `puts` is redirected into the transcript and results/errors use
  distinct generic output kinds.
- The panel/scroll/editor configuration, transcript/input rectangle calculation, input style, and
  `eprompt` presentation route through the reloadable UI library. The stateful `Panel`, `TextEdit`,
  and `ScrollArea` remain instantiated in the host with Molt, history, input/transcript data, and
  effects so their state survives safely.
- The host stores command transcript entries as undecorated scripts. The reloadable adapter owns
  command decoration and the welcome banner instead of baking either into host output strings.
- The dirty sibling fork has been audited and its minimal interpreter crate vendored with its
  BSD-3-Clause license and attribution. Non-library projects and documentation are omitted, and
  `eprompt` remains independent of Molt.
- The first host command bridge queues `zoom fit`, `cursor set/clear`, relative signal focus,
  regex search, undo, and redo, then applies their `ViewerCommand` values after evaluation returns.
  `help` lists this live vocabulary; file/scope/mark/format commands remain to be added.
