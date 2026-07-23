# Commands

All commands assume you're running from the root of the `nano-spec` repository
and that `nano-p4spectec` has been built (see [Installation](./installation.md)).
The spec files live in `nano-p4/spec/*.watsup` and example Nano-P4 programs live
in `nano-p4/testdata/`.

## `elab`

Elaborates the spec and prints the resulting IL.

```shell
$ ./nano-p4spectec elab nano-p4/spec/*.watsup
```

> **Tip:** During active spec development, run `elab` in a watch loop so errors
> surface immediately:
> ```shell
> $ watchexec -w nano-p4/spec ./nano-p4spectec elab nano-p4/spec/*.watsup
> ```

## `parse`

Parses a Nano-P4 source file and prints its IL value. Does not load the spec.

```shell
$ ./nano-p4spectec parse -t \
    -p nano-p4/testdata/positive/action-call.p4 \
    -i nano-p4/include
```

| Flag | Description |
|------|-------------|
| `-p <file>` | Path to the Nano-P4 program |
| `-i <dir>` | Include path for Nano-P4 headers (can be repeated) |
| `-t` | Print the IL value as an indented tree |

## `run`

Elaborates the spec and executes a named relation against a Nano-P4 program.

```shell
$ ./nano-p4spectec run nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -rel Program_ok \
    -p nano-p4/testdata/positive/action-call.p4
```

| Flag | Description |
|------|-------------|
| `-rel <name>` | The relation to invoke (e.g. `Program_ok`, `Program_inst`) |
| `-p <file>` | Path to the Nano-P4 program |
| `-i <dir>` | Include path for Nano-P4 headers (can be repeated) |

On success, prints `passed`. On failure, prints an error message.

## `sim`

Runs an end-to-end simulation against a program and an STF test file.

```shell
$ ./nano-p4spectec sim nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -p nano-p4/testdata/positive/action-call.p4 \
    -stf nano-p4/testdata/positive/action-call.stf
```

| Flag | Description |
|------|-------------|
| `-p <file>` | Path to the Nano-P4 program |
| `-stf <file>` | Path to the STF test file |
| `-i <dir>` | Include path for Nano-P4 headers (can be repeated) |

On success, prints `passed`.

### STF file format

STF (Simple Test Framework) describes packets to inject and the expected output.
Nano-P4 supports two directives:

**`packet <port> <hex-payload>`**: send a packet on the given port.

**`expect <port> <hex-payload>`**: assert a packet is emitted on the given port.
A `packet` with no following `expect` is expected to be dropped.

```
packet 0 010000
expect 0 010000

packet 0 020000

packet 0 030000
expect 0 030000
```

## `test-check`

Batch-typechecks all `.p4` files in one or more directories against `Program_ok`
and prints a per-file `PASS`/`FAIL` summary.

```shell
$ ./nano-p4spectec test-check nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -p4-dir nano-p4/testdata/positive
```

| Flag | Description |
|------|-------------|
| `-p4-dir <dir>` | Directory of `.p4` files to test (can be repeated) |
| `-i <dir>` | Include path for Nano-P4 headers (can be repeated) |
| `-neg` | Negative testing mode. Expect all programs to fail typechecking |

Use `-neg` with a directory of intentionally invalid programs to verify that your
spec correctly rejects them:

```shell
$ ./nano-p4spectec test-check nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -neg \
    -p4-dir nano-p4/testdata/negative
```

## `test-eval`

Batch-runs all `.p4`/`.stf` pairs found in one or more directories and prints a
per-test `PASS`/`FAIL` summary.

```shell
$ ./nano-p4spectec test-eval nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -p4-dir nano-p4/testdata/positive
```

| Flag | Description |
|------|-------------|
| `-p4-dir <dir>` | Directory containing `.p4`/`.stf` pairs (can be repeated) |
| `-i <dir>` | Include path for Nano-P4 headers (can be repeated) |

Only `.p4` files that have a corresponding `.stf` file of the same name are run;
files without a matching `.stf` are silently skipped.

## Quick reference

| Goal | Command |
|------|---------|
| Check spec syntax | `./nano-p4spectec elab nano-p4/spec/*.watsup` |
| Parse a P4 file | `./nano-p4spectec parse -p <file> -t -i nano-p4/include` |
| Type-check a program | `./nano-p4spectec run nano-p4/spec/*.watsup -i nano-p4/include -rel Program_ok -p <file>` |
| Execute with packets | `./nano-p4spectec sim nano-p4/spec/*.watsup -i nano-p4/include -p <file> -stf <stf>` |
| Batch typecheck test | `./nano-p4spectec test-check nano-p4/spec/*.watsup -i nano-p4/include -p4-dir <dir>` |
| Batch execution test | `./nano-p4spectec test-eval nano-p4/spec/*.watsup -i nano-p4/include -p4-dir <dir>` |
