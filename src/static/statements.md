# Statements

This section covers
[`5.05-typing-statement.watsup`](https://github.com/pacokwon/nano-p4-spec/blob/main/5.05-typing-statement.watsup),
which type-checks all statements in Nano-P4.

`Statement_ok` uses a *threading* pattern: each rule takes an
incoming context and produces an outgoing one.
That being said, only variable declarations extend the context, while others simply
pass the context.

```spectec
relation Statement_ok:
  scope typingContext |- statement -| typingContext
  hint(input %0 %1 %2)
```

Read `scope TC_0 |- s -| TC_1` as: "under context `TC_0` at scope `scope`,
statement `s` is well-typed and produces context `TC_1`."

## Empty Statement

```spectec
rule Statement_ok/emptyStatement:
  scope TC |- emptyStatement -| TC
```

An empty statement is always well-typed and leaves the context unchanged.

## Variable Declaration

Variable declarations are handled by a dedicated relation `VarDecl_ok` before
being wrapped by `Statement_ok`:

```spectec
rule VarDecl_ok:
  scope TC_0 |- type name (`= expression) `; -| TC_1
  -- Type_ok: TC_0 |- type ~> typeIR
  -- Expr_ok: scope TC_0 |- expression : typeIR'
  -- Type_eq: typeIR ~~ typeIR'
  -- if id = $id(name)
  -- if varTypeIR = INOUT typeIR
  -- if TC_1 = $add_var_t(scope, TC_0, id, varTypeIR)

rule Statement_ok/variableDeclaration:
  scope TC_0 |- variableDeclaration -| TC_1
  -- VarDecl_ok: scope TC_0 |- variableDeclaration -| TC_1
```

`VarDecl_ok` performs four checks and one update:

1. Elaborate the declared type `type`.
2. The initializer expression is type-checked.
3. Check that both types are equal via `Type_eq`.
4. Compute `id` from the name token.
5. Add the new variable to the context with `$add_var_t`, recording its
   direction as `INOUT`.

The direction `INOUT` is unconditional here: all local variables in Nano-P4 are
read-write by default.
This is distinct from parameters, which carry an explicit direction from their
declaration.

The outgoing context `TC_1` is what the rest of the block sees, so any
subsequent statement can refer to the newly declared variable.

## Assignment Statement

The rule for assignment statements is left as the exercise for this section.

Here is what `Statement_ok/assignmentStatement` should perform:

1. Check that the left-hand side is a valid l-value
2. Check that the right-hand side type-checks.
3. Check that both types are equal.

## Call Statement

There are three kinds of call statements: action call, extern method call, and
table apply method call.

### Action

```spectec
rule Statement_ok/callStatement-name:
  scope TC |- referenceExpression `( argumentList ) `; -| TC
  -- if callableId = $id(referenceExpression)
  -- if ACTION parameterIR* = $find_callableTypeDef_t(TC, callableId)
  -- ArgumentList_ok: scope TC |- argumentList : argumentIR*
  -- Call_convention_ok: parameterIR* `@ argumentIR*
```

An action call like `myAction()` looks up the callable in the context,
extracts its parameter list, type-checks the arguments, and checks the calling
convention.

`ArgumentList_ok` type-checks each argument expression in `argumentList` and
yields `argumentIR*`, where `arugumentIR` is a pair of the argument expression
and its resolved `typeIR`. `Call_convention_ok` verifies that argument directions
and types match the parameters. Both of these relations are covered in the next
section.

### Extern Method

```spectec
rule Statement_ok/callStatement-member:
  scope TC |- (lvalue_base `. member) `( argumentList ) `; -| TC
  -- if expression_base = $expression_of_lvalue(lvalue_base)
  -- Expr_ok: scope TC |- expression_base : typeIR_base
  -- if EXTERN typeId externMethodTypeDefEnv = typeIR_base
  -- if callableId = $id(member)
  -- if VOID callableId `( parameterIR* )
      = $find_map<callableId, externMethodTypeDefIR>(
          externMethodTypeDefEnv,
          callableId
        )
  -- ArgumentList_ok: scope TC |- argumentList : argumentIR*
  -- Call_convention_ok: parameterIR* `@ argumentIR*
```

A method call like `pkt.extract(hdr)` resolves the base expression to an
extern type, looks up the method name in the extern's method environment, and
then checks arguments against the method's parameter list.

`$expression_of_lvalue` converts the syntactic `lvalue_base` into an
`expression` before passing it to `Expr_ok`, since the two are distinct
syntactic sorts.

All extern methods in Nano-P4 return `VOID`, so there is no return type to
propagate.

### Table Apply Method

```spectec
rule Statement_ok/callStatement-table-apply:
  scope TC |- (lvalue_base `. APPLY) `( argumentList ) `; -| TC
  -- if expression_base = $expression_of_lvalue(lvalue_base)
  -- Expr_ok: scope TC |- expression_base : (TABLE typeId)
```

A table apply call like `tbl.apply()` only needs to confirm that the base
expression has a table type.
No argument or parameter checking is needed: table apply takes no arguments in
Nano-P4.

## Block Statement

```spectec
rule Statement_ok/blockStatement:
  scope TC_0 |- blockStatement -| TC_2
  -- if TC_1 = $enter_t(TC_0)
  -- Block_ok: TC_1 |- blockStatement
  -- if TC_2 = $exit_t(TC_1)
```

A block statement pushes a new scope frame with `$enter_t` before checking the
body, then pops it with `$exit_t` afterward.
The outgoing context `TC_2` has the same shape as `TC_0`: any variables
declared inside the block are discarded.

The body is checked by `Block_ok`, which flattens the `statementList` and
threads the context through each statement in sequence:

```spectec
rule Block_ok:
  TC_0 |- `{ statementList }
  -- if statement* = $flatten_statementList(statementList)
  -- Statements_ok: TC_0 |- statement* -| TC_1

rule Statements_ok/nil:
  TC_0 |- eps -| TC_0

rule Statements_ok/cons:
  TC_0 |- statement_h :: statement_t* -| TC_2
  -- Statement_ok: LOCAL TC_0 |- statement_h  -| TC_1
  -- Statements_ok:      TC_1 |- statement_t* -| TC_2
```

`Statements_ok` threads the context left to right: each statement receives the
context produced by the previous one.
Notice that `Statement_ok` is always called with `LOCAL` scope inside
`Statements_ok/cons`, because statements inside a block body live in the local
scope layer.

Also notice that `Block_ok` does not call `$enter_t`/`$exit_t` itself.
The frame push and pop happen in `Statement_ok/blockStatement`, one level up.
`Block_ok` receives a context that already has the new frame on the stack.

## Conditional Statement

```spectec
rule Statement_ok/conditionalStatement:
  scope TC |- IF `( expression ) blockStatement_then
              ELSE blockStatement_else -| TC
  -- Expr_ok: scope TC |- expression : BOOL
  -- Block_ok: TC |- blockStatement_then
  -- Block_ok: TC |- blockStatement_else
```

An `if`/`else` statement checks that the condition has type `BOOL`, then checks
both branches independently under the same incoming context `TC`.
The two branches do not see each other's declarations, and neither escapes to
the enclosing scope: both `Block_ok` premises call `$enter_t`/`$exit_t`
internally (via `Statement_ok/blockStatement`).

The outgoing context is the same `TC` that came in.

## Exercise

**Branch:** `exercise/3.4`

Check out the exercise branch in the `spec` submodule:

```shell
git -C nano-p4/spec checkout exercise/3.4
```

Run the following test to observe the failure:

```shell
./nano-p4spectec check nano-p4/spec/*.watsup -i nano-p4/include -p nano-p4/testdata/exercise/3.4.p4
```

The test program contains a valid assignment statement, but the checker rejects it.

The `Statement_ok/assignmentStatement` rule in
[`5.05-typing-statement.watsup`](https://github.com/pacokwon/nano-p4-spec/blob/main/5.05-typing-statement.watsup)
has been omitted entirely. Write the rule from scratch.

When you are done, restore the original branch:

```shell
git -C nano-p4/spec checkout main
```
