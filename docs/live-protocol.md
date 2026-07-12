# waveview live protocol v1

`waveserve` exposes a WebSocket at `/ws`. Each connection receives:

1. `{"type":"hello","protocol":"waveview-v1"}`
2. `{"type":"reset"}`
3. A binary VCD snapshot
4. Further binary chunks as the input file grows

`reset` means the client must discard buffered bytes. A `resync` message means the client fell
behind and should reconnect for a fresh snapshot. Binary message boundaries have no VCD meaning;
clients must treat them as pieces of one byte stream.

This deliberately transports standard VCD rather than an internal Rust representation. It keeps
simulators and non-Rust producers simple and gives recordings the same semantics as live input.
