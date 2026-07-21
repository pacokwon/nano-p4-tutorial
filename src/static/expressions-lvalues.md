# Expressions and L-values

This section covers the two spec files that type-check expressions and l-values:
[`5.03-typing-expression.watsup`](https://github.com/pacokwon/nano-p4-spec/blob/main/5.03-typing-expression.watsup)
and
[`5.04-typing-lvalue.watsup`](https://github.com/pacokwon/nano-p4-spec/blob/main/5.04-typing-lvalue.watsup).

The relation defined in `5.03` is `Expr_ok`:

```spectec
relation Expr_ok:
  scope typingContext |- expression : typeIR
```

Read `scope TC |- e : T` as: "under context `TC` at scope `scope`, expression `e`
has type `T`."

The `scope` argument threads through many rules so that variable lookup (which
delegates to `$find_var_t`) can consult the right layers of the typing context.

## Literal Expressions

The simplest rules are the literal cases.
Boolean literals always have type `BOOL`:

```spectec
rule Expr_ok/boolean:
  scope TC |- booleanLiteral : BOOL
```

Integer literals carry their width and signedness, and discard their values:

```spectec
rule Expr_ok/integer-unsigned:
  scope TC |- nat W int : BIT `< nat >

rule Expr_ok/integer-signed:
  scope TC |- nat S int : INT `< nat >
```

## Reference Expressions

A reference expression is just a name used as a value:

```spectec
rule Expr_ok/referenceExpression:
  scope TC |- name : typeIR
  -- if id = $id(name)
  -- if _ typeIR = $find_var_t(scope, TC, id)
```

The two premises are:

1. Transform `name` into `id`. `id` is an alias for `text`, and `$id` turns
   a name into text.
2. Look it up via `$find_var_t` and get back a `varTypeIR`, which is a pair
   of `direction` and `typeIR`.
   The direction is wildcarded with `_` because reading a variable has no
   direction constraint.

The `scope` argument to `$find_var_t` is the same one passed into `Expr_ok`,
so lookup follows the correct scope chain automatically.

## Unary Expressions

Nano-P4 has two kinds of unary operators: boolean negation and integer operators.

```spectec
rulegroup Expr_ok/unaryExpression {

  rule Expr_ok/boolean:
    scope TC |- `! expression : BOOL
    -- Expr_ok: scope TC |- expression : BOOL

  rule Expr_ok/integer:
    scope TC |- unop expression : integerTypeIR
    -- if unop <- [ `~, `-, `+ ]
    -- Expr_ok: scope TC |- expression : integerTypeIR

}
```

Both rules check that the operand has the expected type and propagate the same
type to the result.
`!` only works on booleans; `~`, `-`, and `+` work on any integer type
(`BIT<n>` or `INT<n>`), and the result has the same integer type.

The `if unop <- [...]` premise is a *membership check*: it confirms that the
operator in question is one of the listed tokens.

## Binary Expressions

Five groups of binary operators are defined, each with its own rule:

```spectec
rule Expr_ok/arithmetic:
  scope TC |- expression_l binop expression_r : integerTypeIR
  -- if binop <- [ `*, `+, `- ]
  -- Expr_ok: scope TC |- expression_l : integerTypeIR
  -- Expr_ok: scope TC |- expression_r : integerTypeIR
```

Arithmetic operators (`*`, `+`, `-`) require both operands to have the *same*
integer type and produce that type.
The variable `integerTypeIR` appears in all three positions, so pattern matching
enforces that both operands share *exactly the same type*.

```spectec
rule Expr_ok/comparison:
  scope TC |- expression_l binop expression_r : BOOL
  -- if binop <- [ `<=, `>=, ``<, ``> ]
  -- Expr_ok: scope TC |- expression_l : integerTypeIR
  -- Expr_ok: scope TC |- expression_r : integerTypeIR
```

Comparison operators consume integers and produce `BOOL`.
The same variable `integerTypeIR` appears in both premises, so both operands
must have exactly the same integer type, just as with arithmetic operators.

```spectec
rule Expr_ok/equality:
  scope TC |- expression_l binop expression_r : BOOL
  -- if binop <- [ `!=, `== ]
  -- Expr_ok: scope TC |- expression_l : baseTypeIR
  -- Expr_ok: scope TC |- expression_r : baseTypeIR
```

Equality operators accept any base type (`INT<n>`, `BIT<n>`, `BOOL`,
`MATCH_KIND`), not just integers, and return `BOOL`.
The same variable `baseTypeIR` appears in both premises, so both operands must
have exactly the same base type.

```spectec
rule Expr_ok/bitwise:
  scope TC |- expression_l binop expression_r : integerTypeIR
  -- if binop <- [ `&, `^, `| ]
  -- Expr_ok: scope TC |- expression_l : integerTypeIR
  -- Expr_ok: scope TC |- expression_r : integerTypeIR
```

Bitwise operators mirror arithmetic: same-type integers in, same type out.

```spectec
rule Expr_ok/logical:
  scope TC |- expression_l binop expression_r : BOOL
  -- if binop <- [ `&&, `|| ]
  -- Expr_ok: scope TC |- expression_l : BOOL
  -- Expr_ok: scope TC |- expression_r : BOOL
```

Logical operators require both operands to be `BOOL` and return `BOOL`.

## Member Access

Member access is written `expr.field`.
The result type is determined by looking up the field name in the struct or
header definition:

```spectec
rule Expr_ok/struct:
  scope TC |- memberAccessBase `. member : typeIR
  -- Expr_ok: scope TC |- memberAccessBase : typeIR_base
  -- if STRUCT _ `{ (typeIR_field id_field `;)* } = typeIR_base
  -- if id_member = $id(member)
  -- if typeIR = $assoc_<id, typeIR>(id_member, (id_field, typeIR_field)*)

rule Expr_ok/header:
  scope TC |- memberAccessBase `. member : typeIR
  -- Expr_ok: scope TC |- memberAccessBase : typeIR_base
  -- if HEADER _ `{ (typeIR_field id_field `;)* } = typeIR_base
  -- if id_member = $id(member)
  -- if typeIR = $assoc_<id, typeIR>(id_member, (id_field, typeIR_field)*)
```

Both rules follow the same shape:

1. Type-check the base expression and get back a struct or header type `typeIR_base`.
2. Destructure `typeIR_base` to extract its field list `(typeIR_field id_field)*`.
3. Convert the member token to an `id_member`.
4. Look up `id_member` in the field association list with `$assoc_<id, typeIR>`.

The struct and header cases are separate rules because the pattern in step 2
matches either `STRUCT _ { ... }` or `HEADER _ { ... }`, not both.

## Call Expressions

Nano-P4 only supports two kinds of call expressions: parser and control
instantiation via constructor invocation.
These appear in the package argument list (e.g., `NanoSwitch(Parser(), Filter())`).

```spectec
rule Expr_ok/parser:
  scope TC |- (`TID typeId) `( `EMPTY ) : parserObjectTypeIR
  -- if PARSER parameterIR* = $find_callableTypeDef_t(TC, typeId)
  -- if parserObjectTypeIR = PARSER typeId `( parameterIR* )

rule Expr_ok/control:
  scope TC |- (`TID typeId) `( `EMPTY ) : controlObjectTypeIR
  -- if CONTROL parameterIR* = $find_callableTypeDef_t(TC, typeId)
  -- if controlObjectTypeIR = CONTROL typeId `( parameterIR* )
```

Both rules look up `typeId` in the callable type definition environment with
`$find_callableTypeDef_t`. By the time they are called, the parser/control
declaration should have been registered as a callable.
If the callable is a parser, the result is a `PARSER` object type; if it is a
control, the result is a `CONTROL` object type.
Both carry the type name and the parameter list, which are used later when
typechecking the enclosing package instantiation.

The argument list is always `EMPTY` here because parser/control blocks in
Nano-P4 do not have *constructor parameters*, and therefore do not accept
arguments during instantiation.

## Parenthesized Expressions

```spectec
rule Expr_ok/parenthesizedExpression:
  scope TC |- `( expression ) : typeIR
  -- Expr_ok: scope TC |- expression : typeIR
```

Parentheses are transparent: they add no type information and simply propagate
the type of the inner expression.

## L-values

An l-value is an expression that names a storage location and can appear on the
left-hand side of an assignment.
The `Lvalue_ok` relation in `5.04-typing-lvalue.watsup` determines whether
an expression qualifies as an l-value and what type it holds:

```spectec
relation Lvalue_ok:
  scope typingContext |- lvalue : typeIR
```

Only three forms of l-values exist in Nano-P4:

```spectec
syntax lvalue =
  | referenceExpression
  | lvalue `. member
  | `( lvalue )
```

### Reference L-values

```spectec
rule Lvalue_ok/referenceExpression:
  scope TC |- referenceExpression : typeIR
  -- if id = $id(referenceExpression)
  -- if direction typeIR = $find_var_t(scope, TC, id)
  -- if direction = OUT \/ direction = INOUT
```

This rule adds a constraint that `Expr_ok/referenceExpression` does not have:
the direction must be `OUT` or `INOUT`.
`IN` and directionless (`EMPTY`) variables cannot appear on the left of an
assignment because they are read-only.

### Member Access L-values

The rules for member access l-values are left as [an exercise](#exercise). Have fun!

### Parenthesized L-values

```spectec
rule Lvalue_ok/parenthesized:
  scope TC |- `( lvalue ) : typeIR
  -- Lvalue_ok: scope TC |- lvalue : typeIR
```

Like parenthesized expressions, parentheses around an l-value are transparent.

## Exercise

**Branch:** [`exercise/3.3`](https://github.com/pacokwon/nano-p4-spec/tree/exercise/3.3)

Check out the exercise branch in the `spec` submodule:

```shell
git -C nano-p4/spec checkout exercise/3.3
```

Run the following test to observe the failure:

```shell
./nano-p4spectec check nano-p4/spec/*.watsup -i nano-p4/include -p nano-p4/testdata/exercise/3.3.p4
```

The test program assigns to a member of a struct field that is an `INOUT`
parameter.
The checker should accept this, but results in an error instead.

The `Lvalue_ok/structTypeIR` and `Lvalue_ok/headerTypeIR` rules are missing
from `5.04-typing-lvalue.watsup`.

> **Hint:**
> Write them by analogy with `Expr_ok/struct` and `Expr_ok/header` in `5.03-typing-expression.watsup`.

When you are done, restore the original branch:

```shell
git -C nano-p4/spec checkout main
```
