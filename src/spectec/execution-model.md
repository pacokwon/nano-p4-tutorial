# Execution Model

When you run `nano-p4spectec`, two separate artifacts are at play: a
**specification** (your `.watsup` files) and a **program** (a `.p4` source
file).

## Overview

```
  .watsup files          .p4 file
       │                    │
       ▼                    ▼
  ┌─────────┐          ┌─────────┐
  │  elab   │          │  parse  │
  └────┬────┘          └────┬────┘
       │                    │
       ▼                    ▼
   spec (IL)         program (IL value)
       │                    │
       └──────────┬─────────┘
                  │
                  ▼
           ┌─────────────┐
           │ interpreter │
           └──────┬──────┘
                  │
                  ▼
            relation result
         (pass / fail / packets)
```

There are two independent compilation steps, and the interpreter joins their
outputs.

## Step 1: Elaborate the spec

The `.watsup` files are parsed and *elaborated* into *IL* (Internal Language),
a typechecked and desugared representation of the spec that the interpreter
executes directly.

*Elaboration* checks that the spec itself
is well-formed: syntax definitions are consistent, rule conclusions match their
relation signatures, function clauses are well-typed, and so on.

You can run this step in isolation with:

```shell
$ ./nano-p4spectec elab nano-p4/spec/*.watsup
```

## Step 2: Parse the program

The `.p4` source file is parsed by the Nano-P4 parser into a **P4-SpecTec IL
value**. It is a tree of constructor tags and nested values that directly mirrors
the `syntax` definitions in the spec. Because it is a value in the same language
that the spec is written in, the interpreter can pass it directly to spec relations.

You can inspect this value with:

```shell
$ ./nano-p4spectec parse -p <file> -i nano-p4/include
```

The `-t` flag prints it as an indented tree, which is easier to read:

```shell
$ ./nano-p4spectec parse -t -p <file> -i nano-p4/include
```

<details>
<summary>Example output</summary>

```
$ cat action.p4
action MyAction() {
    bit<8> x = 8w42;
}

$ ./nano-p4spectec parse \
    -i nano-p4/include \
    -p action.p4 \
    -t
program % %
├── declarationList /* empty */
└── actionDeclaration ACTION % (%) %
    ├── identifier `ID %
    │   └── "MyAction"
    ├── parameterList /* empty */
    └── blockStatement {%}
        └── statementList % %
            ├── statementList /* empty */
            └── variableDeclaration % % % ;
                ├── baseType BIT <%>
                │   └── +8
                ├── identifier `ID %
                │   └── "x"
                └── initializer = %
                    └── integerLiteral % W %
                        ├── 8
                        └── +42
```
</details>

## Step 3: Interpret

The interpreter takes the elaborated spec and the program value and executes a
relation against the program. The relation is specified with `-rel`:

```shell
$ ./nano-p4spectec run nano-p4/spec/*.watsup \
    -i nano-p4/include \
    -rel Program_ok \
    -p <file>
```

For example, this command takes a `.p4` program, converts it to an IL value, and
passes it to the `Program_ok` relation. The `Program_ok` relation typechecks an
entire program. Therefore, we are executing the typechecker against a program.

## What this means in practice

- A **spec error** (syntax, type, or rule error in `.watsup`) surfaces during
  elaboration, before the program is touched.
- A **parse error** means the `.p4` file is not valid Nano-P4 syntax.
- A **runtime error** means the interpreter got stuck executing the spec against
  the program.
  This is typically due to a rule that has no matching case for the given input.
- A **test failure** from `sim` means the spec's dynamic semantics produced
  different output packets than the STF file expected.
