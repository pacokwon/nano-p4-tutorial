# Types

In this section we explore how the type checker reasons about types.
Two things in particular are of special interest:
**well-formedness** and **equivalence**.

The type checker needs to know two things about any type it encounters:
whether the type is well-formed, and whether two types are the same.
These two jobs are handled by two relations defined in
[`5.02-typing-type.watsup`](https://github.com/pacokwon/nano-p4-spec/blob/main/5.02-typing-type.watsup).

## Type Elaboration: `Type_ok`

The `Type_ok` relation *resolves* a surface-syntax type into its internal
representation (`typeIR`).

```spectec
relation Type_ok:
  typingContext |- type ~> typeIR
  hint(input %0 %1)
```

Read `typingContext |- type ~> typeIR` as: "under `typingContext`,
the type `type` elaborates to the internal type `typeIR`."

For base types, the rule is trivial: the surface type is already its own
internal representation.

```spectec
rule Type_ok/signed:   TC |- INT `< n >    ~> INT `< n >
rule Type_ok/unsigned: TC |- BIT `< n >    ~> BIT `< n >
rule Type_ok/boolType: TC |- BOOL          ~> BOOL
rule Type_ok/matchKindType: TC |- MATCH_KIND ~> MATCH_KIND
```

Named types require a lookup.

```spectec
rule Type_ok/typeName:
  TC |- (`TID typeId) ~> typeIR
  -- if typeDefIR = $find_typeDef_t(TC, typeId)
  -- if typeIR = $typeIR_of_typeDefIR(typeDefIR)
```

When the checker sees a named type such as `Header`, it:

1. Looks up `Header` in the typing context `TC` via `$find_typeDef_t`,
   retrieving a `typeDefIR`.
2. Converts that `typeDefIR` to a `typeIR` with `$typeIR_of_typeDefIR`.

`$typeIR_of_typeDefIR` is a two-clause function defined in `5.00`:

```spectec
def $typeIR_of_typeDefIR(dataTypeIR)       = dataTypeIR
def $typeIR_of_typeDefIR(objectTypeDefIR)  = objectTypeDefIR
```

Both clauses are identity-like: a `typeDefIR` is either a `dataTypeIR` or an
`objectTypeDefIR`, and both are subtypes of `typeIR`, so the conversion just
changes the tag.
The function exists to make the type explicit to the elaboration rule.

When a named type is resolved into its underlying type, not only do we know that
it is a valid type, but the resolved information can be used somewhere else
along the type checking process.

### What `typeIR` looks like

Understanding elaboration requires knowing what the internal type universe
looks like.

**Base types** need no further structure:

```spectec
syntax baseTypeIR =
  | INT `< nat >
  | BIT `< nat >
  | BOOL
  | MATCH_KIND
```

**Data types** are user-defined and carry their full field list:

```spectec
syntax structTypeIR = STRUCT typeId `{ fieldTypeIR* }
syntax headerTypeIR = HEADER typeId `{ fieldTypeIR* }
```

Note that structs and headers carry both a `typeId` (the name the programmer
gave them) and the full field list `fieldTypeIR*`.
The name will matter for equality, as we will see below.

**Object types** represent instantiable components (parsers, controls,
packages, externs, tables):

```spectec
syntax parserObjectTypeIR  = PARSER  typeId `( parameterIR* )
syntax controlObjectTypeIR = CONTROL typeId `( parameterIR* )
syntax packageObjectTypeIR = PACKAGE typeId `( parameterIR* )
syntax externObjectTypeIR  = EXTERN  typeId externMethodTypeDefEnv
syntax tableObjectTypeIR   = TABLE   typeId
```

Each carries its name and its parameter list (or method map, for externs).

## Type Equality: `Type_eq`

`Type_eq` decides whether two internal types are the same.

```spectec
relation Type_eq:
  typeIR ~~ typeIR
  hint(input %0 %1)
```

Read `typeIR_a ~~ typeIR_b` as: "`typeIR_a` and `typeIR_b` are equal types."

### Base types

```spectec
rule Type_eq/baseTypeIR:
  baseTypeIR ~~ baseTypeIR
```

Two base types are equal if and only if they are *syntactically identical*.
`INT<8>` is not equal to `INT<16>`, and `BOOL` is not equal to `BIT<1>`.
Pattern matching handles this: the same variable `baseTypeIR` appears on both
sides, so the rule only fires when the two sides are the same term.

### Structs and headers

```spectec
rule Type_eq/structTypeIR:
  (STRUCT typeId `{ _ }) ~~ (STRUCT typeId `{ _ })

rule Type_eq/headerTypeIR:
  (HEADER typeId `{ _ }) ~~ (HEADER typeId `{ _ })
```

Two struct/header types are equal when they share the same `typeId`.
The field lists on both sides are wildcarded with `_` and ignored entirely.

This is *nominal equality*, not structural equality.
If two independent structs happen to have identical fields but different names,
they are not considered equal by this relation.

Nominal equality is sound here because Nano-P4 uses a single global type
definition environment: every type name maps to exactly one definition for the
entire program.

If `typeId` is the same, the structs behind it are definitionally the same
struct, so comparing names is sufficient. This is because there is one global
type definition environment in Nano-P4.

### Externs

```spectec
rule Type_eq/externObjectTypeIR:
  (EXTERN typeId _) ~~ (EXTERN typeId _)
```

Extern types are also compared by name only.
As with structs, the global type environment guarantees that the same name
always resolves to the same extern declaration.

### Parsers, controls, and packages

```spectec
rule Type_eq/parserObjectTypeIR:
  (PARSER _ `( parameterIR_a* )) ~~ (PARSER _ `( parameterIR_b* ))
  -- (ParameterType_eq: parameterIR_a ~~ parameterIR_b)*

rule Type_eq/controlObjectTypeIR:
  (CONTROL _ `( parameterIR_a* )) ~~ (CONTROL _ `( parameterIR_b* ))
  -- (ParameterType_eq: parameterIR_a ~~ parameterIR_b)*

rule Type_eq/packageObjectTypeIR:
  (PACKAGE _ `( parameterIR_a* )) ~~ (PACKAGE _ `( parameterIR_b* ))
  -- (ParameterType_eq: parameterIR_a ~~ parameterIR_b)*
```

Parsers, controls, and packages are compared *structurally*: their names
(the `_`) are ignored, and equality holds when the parameter lists are equal
pairwise under `ParameterType_eq`.

This is the opposite choice from how we compare structs/headers.
For structs and headers, a type name is a declaration: two values have the same
type only if they were declared under that exact name.
Parsers and controls are different: a control type declaration describes an
*interface*, and any control block that satisfies that interface is a valid
implementation.
The name of the implementing block is irrelevant; what matters is that its
parameter list matches.

Consider:

```p4
// control type declaration (in nano_model.p4)
control filter(inout Header hdr, out bool accept);

// control block declaration (user-written)
control Filter(inout Header hdr, out bool pass) {
    apply { pass = true; }
}

// package instantiation
NanoSwitch(MyParser(), Filter()) main;
```

`filter` and `Filter` are two different names, yet `Filter` is a valid
implementation of the `filter` interface because their parameter lists match.
Structural comparison is what captures this.

### Parameter equality

All three iterated premises above delegate to `ParameterType_eq`:

```spectec
rule ParameterType_eq:
  (direction typeIR_a _) ~~ (direction typeIR_b _)
  -- Type_eq: typeIR_a ~~ typeIR_b
```

Two parameters are equal when they share the same *direction* and have equal
types under `Type_eq`.
The parameter name (the trailing `_`) is irrelevant for equality.

### Tables

```spectec
rule Type_eq/tableObjectTypeIR:
  (TABLE typeId) ~~ (TABLE typeId)
```

Tables are compared by name, consistent with the other data types.

## Exercise

**Branch:** [`exercise/3.2`](https://github.com/pacokwon/nano-p4-spec/tree/exercise/3.2)

Check out the exercise branch in the `spec` submodule:

```shell
git -C nano-p4/spec checkout exercise/3.2
```

Run the following test to observe the failure:

```shell
./nano-p4spectec check nano-p4/spec/*.watsup -i nano-p4/include -p nano-p4/testdata/exercise/3.2.p4
```

The test program declares two variables with the same struct type and tries to
assign one to the other.
The checker should accept this, but it rejects it.

Find the bug/missing rule in `5.02-typing-type.watsup` and fix it accordingly.

When you are done, restore the original branch:

```shell
git -C nano-p4/spec checkout main
```
