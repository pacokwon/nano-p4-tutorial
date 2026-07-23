# Test Suite

A test suite for Nano-P4 lives in `nano-p4/testdata/` and is split into two directories:

- `positive/`: 32 nano-p4 programs that are expected to type-check and produce
               correct output
- `negative/`: 21 nano-p4 programs that are expected to be rejected by the type
               checker

## Positive tests

Each positive test is a pair of files with the same base name:

- `<name>.p4`: a valid Nano-P4 program
- `<name>.stf`: a packet test that drives the program and checks its output

For example, `table-const-entries.p4` installs an ACL table and exercises it
with packets of different packet types:

```p4
#include <nano_model.p4>

action drop(out bool pass) {
    pass = false;
}

action fwd(out bool pass) {
    pass = true;
}

parser Parser(packet_in pkt, out Header hdr) {
    state start {
        pkt.extract(hdr.nanonet);
        transition accept;
    }
}

control Filter(inout Header hdr, out bool pass) {
    table acl {
        key = { hdr.nanonet.packetType : exact; }
        actions = { drop(pass); fwd(pass); }
        const entries = {
            (7w1) : fwd(pass);
            (7w2) : fwd(pass);
            (7w0) : drop(pass);
        }
    }
    apply {
        pass = true;
        acl.apply();
    }
}

NanoSwitch(Parser(), Filter()) main;
```

Its companion `table-const-entries.stf` sends three packets and asserts which ones
are forwarded:

```
packet 0 010000
expect 0 010000

packet 0 000000

packet 0 050000
expect 0 050000
```

The first packet (packetType = 0x00) matches entry `7w0` and
is dropped, hence the absence of a corresponding `expect` directive.
The second packet (packetType = 0x01) matches entry `7w1` and is forwarded.
The third packet (packetType = 0x05) has no matching entry. `pass` stays
`true` from the default and the packet is forwarded.

## Negative tests

Negative tests are `.p4` files only. There is no `.stf` companion because the
program should not get past typechecking.

For example, `bit-arith-mixed-widths.p4` attempts arithmetic between operands
of different widths:

```p4
#include <nano_model.p4>

parser Parser(packet_in pkt, out Header hdr) {
    state start {
        transition accept;
    }
}

control Filter(inout Header hdr, out bool pass) {
    apply {
        bit<8> x = 8w10;
        bit<16> y = 16w3;
        bit<8> sum = x + y;
        pass = sum == 8w13;
    }
}

NanoSwitch(Parser(), Filter()) main;
```

This should be rejected because `x + y` mixes `bit<8>` and `bit<16>`, which the
type system does not allow.

## Running the suite

Run all positive tests (type-check + simulate):

```shell
$ ./nano-p4spectec test-check nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -p4-dir nano-p4/testdata/positive

$ ./nano-p4spectec test-eval nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -p4-dir nano-p4/testdata/positive
```

Run all negative tests (expect rejection):

```shell
$ ./nano-p4spectec test-check nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -neg \
    -p4-dir nano-p4/testdata/negative
```

As you write new spec rules, re-running these commands is the primary way
to check correctness.
