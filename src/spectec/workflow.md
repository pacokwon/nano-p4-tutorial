# Workflow

This section covers the common commands you'll use when working with the Nano-P4
spec via the `nano-p4spectec` tool. All commands assume you're running from the
root of the `nano-spec` repository and that `nano-p4spectec` has been built
(see [Installation](./installation.md)).

The spec files live in `nano-p4/spec/*.watsup` and example Nano-P4 programs live
in `nano-p4/testdata/`.

## `elab`: Elaborate the Spec

The `elab` command parses and *elaborates* the specification files,
printing the resulting IL. This is the quickest way to check that your `.watsup`
files are syntactically valid and well-typed.

```shell
$ ./nano-p4spectec elab nano-p4/spec/*.watsup

... (truncated output) ...
-- output: % |- % : forwardingDecision -| EC_2
```

If elaboration succeeds, the IL is printed to stdout. Errors are reported with
file and line information.

> **Tip:** During active spec development, run `elab` in a watch loop so errors
> surface immediately. Here's an example using [watchexec](https://github.com/watchexec/watchexec):
> ```shell
> $ watchexec -w nano-p4/spec ./p4spectec elab nano-p4/spec/*.watsup
> ```

## `parse`: Parse a Nano-P4 Program

The `parse` command parses a Nano-P4 source file and prints its AST. It does
not load the spec, so it is useful for quickly checking whether a `.p4` file is
syntactically valid.

```shell
$ ./nano-p4spectec parse -t \
    -p nano-p4/testdata/positive/action-call.p4 \
    -i nano-p4/include
```

| Flag | Description |
|------|-------------|
| `-p <file>` | Path to the Nano-P4 program |
| `-i <dir>` | Include path for Nano-P4 headers (can be repeated) |
| `-t` | Print the AST as a tree instead of a linear value |

## `run`: Run a Relation Against a Program

The `run` command elaborates the spec and then executes a pre-defined relation against
a Nano-P4 program. Use this to typecheck or interpret a program using rules
defined in the spec.

```shell
$ ./nano-p4spectec run nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -rel Program_ok \
    -sl \
    -p nano-p4/testdata/positive/action-call.p4
```

| Flag | Description |
|------|-------------|
| `-rel <name>` | The relation to invoke (e.g. `Program_ok`, `Program_inst`) |
| `-p <file>` | Path to the Nano-P4 program |
| `-i <dir>` | Include path for Nano-P4 headers (can be repeated) |

On success, `run` prints `passed`. On failure it prints a syntax or runtime
error message.

`Program_ok` is the name of the relation for typechecking the entire program.

## `sim`: Simulate a Nano-P4 Program

The `sim` command runs a full end-to-end simulation of the Nano-P4 switch model
against a program and an STF (Simple Test Framework) test file. STF files
describe packets to send and the expected output.

```shell
$ ./nano-p4spectec sim nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -sl \
    -p nano-p4/testdata/positive/action-call.p4 \
    -stf nano-p4/testdata/positive/action-call.stf
```

| Flag | Description |
|------|-------------|
| `-p <file>` | Path to the Nano-P4 program |
| `-stf <file>` | Path to the STF test file |
| `-i <dir>` | Include path for Nano-P4 headers (can be repeated) |
| `sl` / `il` | Interpreter mode (default: `sl`) |

On success, `sim` prints `passed`. The STF file drives packet injection and
checks the output packets against expected values.

### STF file format

STF (Simple Test Framework) is a lightweight packet-level test format: you
describe what packets to inject and what output to expect, and the test framework
compares the two. Nano-P4 supports two directives: `packet` and `expect`.

**`packet <port> <hex-payload>`**: sends a packet on the given port with the
given hex-encoded payload.

**`expect <port> <hex-payload>`**: asserts that a packet with the given
hex-encoded payload is emitted on the given port after processing the preceding
`packet`. If a `packet` has no following `expect`, the packet is expected to be
dropped (i.e. no output is produced).

For example:

```
packet 0 010000
expect 0 010000

packet 0 020000

packet 0 030000
expect 0 030000
```

This sends three packets on port 0. The first and third are expected to pass
through unchanged; the second is expected to be dropped.

## Quick Reference

| Goal | Command |
|------|---------|
| Check spec syntax | `./nano-p4spectec elab nano-p4/spec/*.watsup` |
| Parse a P4 file | `./nano-p4spectec parse -p <file> -t -i nano-p4/include` |
| Typecheck a program | `./nano-p4spectec run nano-p4/spec/*.watsup -i nano-p4/include -rel Program_ok -sl -p <file>` |
| Simulate with packets | `./nano-p4spectec sim nano-p4/spec/*.watsup -i nano-p4/include -sl -p <file> -stf <stf>` |
