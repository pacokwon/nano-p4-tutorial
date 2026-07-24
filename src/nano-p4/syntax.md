# Syntax Definition in P4-SpecTec

With the scope of Nano-P4 in mind, let us see how its syntax is expressed in
P4-SpecTec. The full syntax is defined in
[`1-syntax.watsup`](https://github.com/pacokwon/nano-p4-spec/blob/main/1-syntax.watsup).
Reading it will also give you a feel for what P4-SpecTec `syntax` definitions
look like at scale before we move into the type-checking and evaluation rules.

## Terminals and Non-terminals

Every `syntax` production in P4-SpecTec is built from two kinds of atoms.

**Non-terminals** are references to other `syntax` productions. They appear as
lowercase names, for example `expression`, `type`, or `name`.

**Terminals** are concrete tokens of the language being specified. They appear
in two forms:

- **Keyword terminals** are written in ALL_CAPS: `IF`, `ELSE`, `STRUCT`,
  `PARSER`, `CONTROL`, and so on. These correspond to reserved keywords in
  Nano-P4.
- **Punctuation terminals** are written with a leading backtick: `` `( ``,
  `` `) ``, `` `{ ``, `` `} ``, `` `; ``, `` `. ``, `` `= ``, and so on. The
  backtick distinguishes a literal token from the syntax of P4-SpecTec.

For example, the production for an assignment statement:

```spectec
syntax assignmentStatement = lvalue `= expression `;
```

reads as: an assignment is an `lvalue` non-terminal, followed by the literal `=`
token, followed by an `expression` non-terminal, followed by the literal `;`
token.

## Literals

The two literal forms in Nano-P4 are booleans and integers.

```spectec
syntax booleanLiteral =
  | TRUE
  | FALSE

syntax integerLiteral =
  | nat W int
  | nat S int
```

`TRUE` and `FALSE` are terminals. Integer literals carry two pieces of metadata:
a width `nat` and a value `int`. The `W` terminal marks an unsigned bit-string
and `S` a signed integer. These correspond to `bit<N>` and `int<N>` literals in
source Nano-P4.

## Identifiers

Nano-P4 uses two distinct identifier categories:

```spectec
syntax identifier = `ID text
syntax typeIdentifier = `TID text
```

`identifier` carries the `ID` tag and a `text` payload; `typeIdentifier` uses
`TID`. The split mirrors the official P4 grammar, where the lexer distinguishes
regular identifiers from type names it has already seen.

From these two primitives, several name non-terminals are derived:

```spectec
syntax nonTypeName =
  | identifier
  | APPLY | KEY | ACTIONS | STATE

syntax typeName = typeIdentifier

syntax name = nonTypeName
```

`nonTypeName` adds the contextual keywords `APPLY`, `KEY`, `ACTIONS`, and
`STATE` as valid identifiers: they are reserved in some positions but can appear
as plain names in others. `typeName` is just a `typeIdentifier` wrapped for
clarity. `name` collapses to `nonTypeName`, which is what most of the spec
refers to.

## Types

```spectec
syntax integerType =
  | BIT `< int >
  | INT `< int >

syntax baseType =
  | integerType
  | BOOL
  | MATCH_KIND

syntax type =
  | baseType
  | namedType
```

`integerType` captures `bit<N>` and `int<N>` with the width stored as an `int`
meta-value directly in the syntax tree. `baseType` bundles integer types with
the two keyword types `BOOL` and `MATCH_KIND`. `type` is the union of base types
and named types (structs and headers resolved by name).

## Parameters

```spectec
syntax parameter =
  direction type name

syntax direction = `EMPTY | IN | OUT | INOUT
```

A parameter is a direction, a type, and a name, laid out in sequence.
`direction` has four cases: the three P4 keywords and `` `EMPTY `` for the
directionless case (parameters with no direction annotation).

The `parameterList` production handles the empty-or-nonempty split:

```spectec
syntax parameterList =
  | `EMPTY
  | nonEmptyParameterList
```

The `` `EMPTY `` here is a P4-SpecTec internal sentinel, not a P4 keyword. It is
a terminal token in the grammar, but it will never appear in a real Nano-P4
source file.

```spectec
syntax nonEmptyParameterList =
  | parameter
  | nonEmptyParameterList `, parameter
```

Any sequence of productions such as `nonEmptyParameterList` is represented in
left-recursive form to reflect the Yacc/Bison style of grammar used in P4.

However, it is necessary to convert these to right recursive lists to access
elements from left to right. Therefore, alongside the syntax, the spec defines a
helper function to flatten a `parameterList` into a flat list `parameter*`:

```spectec
dec $flatten_parameterList(parameterList) : parameter*
def $flatten_parameterList(`EMPTY) = eps
def $flatten_parameterList(parameter) = [ parameter ]
def $flatten_parameterList(nonEmptyParameterList `, parameter)
  = $flatten_parameterList(nonEmptyParameterList) ++ [ parameter ]
```

This pattern, a `dec` / `def` pair that recursively accumulates elements into a
list, appears throughout `1-syntax.watsup` for every list-valued production:
`nameList`, `argumentList`, `statementList`, and so on. The type checker and
evaluator call these helpers instead of pattern-matching on the recursive list
syntax directly.

## Expressions

Expressions form the most layered part of the grammar. The spec defines them in
named groups before assembling them under `expression`:

```spectec
syntax expression =
  | literalExpression
  | referenceExpression
  | unaryExpression
  | binaryExpression
  | memberAccessExpression
  | callExpression
  | parenthesizedExpression
```

A few sub-productions are worth noting.

**Unary and binary expressions** encode their operators as separate `syntax`
productions:

```spectec
syntax unop = `! | `~ | `- | `+

syntax binop =
  | `* | `+ | `-
  | `<= | `>= | ``< | ``> | `!= | `==
  | `& | `^ | `| | `&& | `||
```

**Member access** and **call** expressions use forward-declared non-terminals to
break the mutual recursion between `expression`, `memberAccessBase`, and
`callTarget`:

```spectec
syntax memberAccessBase           ;; forward declaration

syntax memberAccessExpression = memberAccessBase `. member
syntax callExpression = callTarget `( argumentList )

;; ... expression is now fully defined ...

syntax memberAccessBase = expression
syntax callTarget = namedType
```

P4-SpecTec requires a `syntax` declaration before first use, so these are
declared with no alternatives and then given their full definition later in the
file after `expression` itself is complete.

## L-values

```spectec
syntax lvalue =
  | referenceExpression
  | lvalue `. member
  | `( lvalue )
```

L-values are a strict subset of expressions: a variable reference, a member
access rooted at an l-value, or a parenthesized l-value. The spec keeps `lvalue`
separate from `expression` so that the type checker can enforce assignment rules
without inspecting the expression structure at every call site.

## Statements

```spectec
syntax statement =
  | emptyStatement
  | variableDeclaration
  | assignmentStatement
  | callStatement
  | blockStatement
  | conditionalStatement
```

`conditionalStatement` requires both branches:

```spectec
syntax conditionalStatement =
  IF `( expression ) blockStatement ELSE blockStatement
```

This directly encodes the Nano-P4 restriction noted in the scope section: `if`
without `else` is not allowed.

Unlike P4, `variableDeclaration` _requires_ an initializer:

```spectec
syntax initializer = `= expression

syntax variableDeclaration =
  type name initializer `;
```

## Declarations

Nano-P4 has the following top-level declaration forms, each with its own
production:

```spectec
syntax declaration =
  | instantiation
  | actionDeclaration
  | matchKindDeclaration
  | externDeclaration
  | parserDeclaration
  | controlDeclaration
  | typeDeclaration
```

A few representative examples:

```spectec
syntax actionDeclaration =
  ACTION name `( parameterList ) blockStatement

syntax externObjectDeclaration =
  EXTERN name `{ externMethodPrototypeList }

syntax controlDeclaration =
  CONTROL name
    `( parameterList )
    `{ controlLocalDeclarationList APPLY controlBody }
```

`controlDeclaration` places `APPLY` inside the body braces, which matches the
actual P4 syntax.

Parser and control declarations follow the split-body pattern seen in the scope
section: local declarations come first, then states (for parsers) or the apply
body (for controls).

## Putting It Together

At the top level, a Nano-P4 program is a sequence of declarations:

```spectec
syntax program =
  | `EMPTY
  | program declaration
```

As with all list productions, a `$flatten_program` helper converts it to a flat
`declaration*` that the rest of the spec consumes.

With this syntax definition in hand, the type checker and evaluator can refer to
every Nano-P4 construct by name, and the spec stays readable as the rules grow
more complex in the chapters ahead.
