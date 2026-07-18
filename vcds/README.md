# Local VCD fixtures

VCD files in this directory are ignored because real captures can be large or proprietary. Keep
small, redistributable regression fixtures here only when they are explicitly force-added.

`edge_cases.vcd` is the checked-in parser and renderer regression fixture. It contains scalar and
vector values, all four logic states, a repeated value change, and a final timestamp with no value
change so tests can verify that signals extend to the end of the capture. Its waveform commands are
mirrored in the self-contained parser test so an uncommitted fixture does not break Git-backed Nix
flake evaluation.
