# NanoSwitch Architecture

NanoSwitch is the target architecture for Nano-P4.
It is a minimal packet-filtering pipeline heavily inspired by the [eBPF architecture](https://p4lang.github.io/p4c/ebpf_backend.html).

## Package declaration

The NanoSwitch architecture is declared in `nano_model.p4`:

```p4
parser parse(packet_in packet, out Header hdr);
control filter(inout Header hdr, out bool accept);

package NanoSwitch(parse p, filter f);
```

A Nano-P4 program must instantiate this package at the top level, providing a
concrete parser and filter:

## Pipeline

```
                 ┌──────────────────────────────────────────────┐
                 │               NanoSwitch                     │
                 │                                              │
  packet_in      │   ┌────────┐   accept   ┌────────┐           │
─────────────────┼──►│ Parser ├───────────►│ Filter ├───────┐   │
                 │   └────────┘            └────────┘       │   │
                 │        │                    │            │   │
                 │        │ reject             │            ▼   │
                 │        │                   accept?  forward/drop
                 │        ▼                             │       │
                 │       DROP                           └───────┼──► packet_out
                 │                                              │    (or dropped)
                 └──────────────────────────────────────────────┘
```

The pipeline has two stages:

1. **Parser**: Reads the packet and extracts the `Header`.
2. **Filter**: Receives the parsed header and an `accept` flag (initialized to `false`).
   The control block sets `accept` based on header fields and table lookups.
   After the filter runs, the pipeline reads `accept` and either forwards or drops
   the packet.

Unlike more complex architectures found in the P4 ecosystem, there are no inter-block
behaviors or logic in the NanoSwitch architecture.

```p4
NanoSwitch(MyParser(), MyFilter()) main;
```

## Core definitions

The architecture also provides a fixed set of core definitions in `nano_core.p4`.

### The Nanonet header

```p4
header Nanonet {
    bool     drop;
    bit<7>   packetType;
    bit<8>   src;
    bit<8>   dst;
}
```

This is the only header type in NanoSwitch. It is 24 bits (3 bytes) wide.
The `drop` field is the primary signal used by filtering programs. Setting it
drives the forwarding decision in typical programs, though the actual decision
is controlled by the `accept` flag passed to the filter.

### The Header struct

```p4
struct Header {
    Nanonet nanonet;
}
```

The top-level header struct holds exactly one `Nanonet` header.
This is what the parser extracts into and what the filter operates on.

### The packet_in extern

```p4
extern packet_in {
    void extract(out Nanonet hdr);
}
```

`packet_in` is the only extern object available to the parser.
Calling `extract` reads the next 24 bits from the incoming packet into the
provided header. If fewer than 24 bits remain, the call is a no-op, and the
cursor retains its position.

### Built-in action and match kind

```p4
action NoAction() {}

match_kind { exact }
```

`NoAction` is a no-op action available for use in table action lists.
`exact` is the only supported match kind for table key fields.
