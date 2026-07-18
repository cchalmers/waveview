# Stage 7: Formats, loading, live data, and performance

## Outcome

Make waveview reliable on realistic captures and long-running live streams while preserving its UI
and Vim identity.

## Backend decision

Benchmark the existing parser/storage against `wellen` using small, large, sparse, and vector-heavy
captures. Measure:

- Header discovery latency.
- Time until hierarchy is usable.
- Time until first selected signal is visible.
- Peak memory.
- Transition lookup latency for cursor motion.
- Incremental/live suitability.

Adopt `wellen` if it materially improves lazy access, FST/GHW support, and large-file behavior. Keep
waveview-owned signal IDs, hierarchy views, commands, and rendering APIs so the backend remains
replaceable. Do not copy Surfer's container architecture wholesale.

## File loading

- [ ] Parse/load away from the UI thread with progress and cancellation.
- [ ] Make hierarchy available after the header when the backend permits it.
- [ ] Load transition bodies lazily for displayed signals.
- [ ] Add file watching and explicit reload/switch while preserving displayed items, marks, and
  viewport where possible.
- [ ] Add FST and GHW only after the backend abstraction and VCD regression suite are stable.

## Rendering performance

- Cull transitions and rows outside the visible viewport.
- Coalesce transitions that occupy the same pixel without changing cursor lookup correctness.
- Cache geometry by signal, viewport quantization, format, and style where profiling supports it.
- Keep analog rendering and complex translators deferred until digital traces are consistently
  responsive.
- Add repeatable benchmarks rather than relying on continuous repaint FPS.

## Live VCD

- [ ] Stop reparsing the entire accumulated stream after every message.
- [ ] Maintain incremental parser state or define framed header/change protocol messages.
- [ ] Add reconnect, health/status, bounded buffering, backpressure, and clear error states.
- [ ] Preserve focus, displayed items, marks, and follow/manual viewport modes during updates.
- [ ] Define what truncation/restart of the producing VCD means to the viewer.
- [ ] Keep transport and Tcl websocket facilities separate; viewer scripting may request a
  connection but does not implement the protocol itself.

## Acceptance criteria

- Performance budgets and representative fixtures are checked or at least reported in CI.
- Large files do not block input for the whole parse.
- Transition navigation remains fast at high zoom and with long simulations.
- Live viewing runs for an extended test without unbounded memory growth or whole-file reparsing.
- Backend replacement does not change Vim commands, session format concepts, or UI component APIs.
