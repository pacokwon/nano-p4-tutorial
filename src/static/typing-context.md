# Typing Context

Before writing a single typing rule, we need somewhere to store what the checker
knows so far: which type names exist, which actions and parsers have been
declared, and which variables are in scope. That storage is the _typing
context_, defined in
[`5.00-typing-context.watsup`](https://github.com/pacokwon/nano-p4-spec/blob/main/5.00-typing-context.watsup).

## The Three Layers

The typing context is not a single flat map. It is split into three _layers_,
each serving a different scoping purpose.

```spectec
syntax typingContext =
  { GLOBAL globalTypingLayer,
    BLOCK  blockTypingLayer,
    LOCAL  localTypingLayer }
```

**Global layer** holds information that is visible everywhere in the program:
type definitions (structs, headers, externs, parsers, controls, packages),
callable definitions (actions, parsers, controls), and global variables.

```spectec
syntax globalTypingLayer =
  { TYPE     typeDefEnv,
    CALLABLE callableTypeDefEnv,
    FRAME    typeFrame }
```

**Block layer** holds the parameters of the current parser or control block.
Parameters are declared once at the top of a block and visible throughout it,
but they must be kept separate from local variables so that scoping rules can be
enforced correctly. This layer also holds variables declared at block level
declarations in parser and control blocks.

```spectec
syntax blockTypingLayer =
  { FRAME typeFrame }
```

**Local layer** holds local variables declared inside a block body. It is a
_stack_ of frames rather than a single frame, because P4 allows nested block
statements, each of which introduces its own scope.

```spectec
syntax localTypingLayer =
  { FRAMES typeFrame* }
```

A `typeFrame` is a map from variable names to their types and directions:

```spectec
syntax varTypeIR  = direction typeIR
syntax typeFrame  = map<id, varTypeIR>
```

The direction (`IN`, `OUT`, `INOUT`, or `EMPTY` for directionless) is stored
alongside the type because the checker needs it to enforce l-value rules, such
as only `OUT` and `INOUT` variables being allowed on the left side of an
assignment.

## Callables and Type Definitions

Two environments in the global layer deserve a closer look.

`typeDefEnv` maps type names to their internal representations (`typeDefIR`).
When the checker sees a named type like `Header`, it looks it up here to resolve
it to its full struct or header definition.

`callableTypeDefEnv` maps callable names to their callable type definitions:

```spectec
syntax callableTypeDef =
  | ACTION parameterIR*
  | PARSER parameterIR*
  | CONTROL parameterIR*
```

This is what the checker looks up when it sees an action call or a constructor
invocation such as `MyAction()` or `Filter()`.

## The `scope` Tag

Many functions in this file take a `scope` argument:

```spectec
syntax scope = GLOBAL | BLOCK | LOCAL
```

This tag is how the spec selects which layer to read from or write to. Rather
than writing three separate functions for each operation, the spec uses one
function with three pattern-matched cases dispatching on the scope. You will see
this pattern throughout the static semantics.

## Frame Entry and Exit

When the checker enters a block statement, it pushes a new empty frame onto the
local stack. When it exits, it pops that frame and discards any variables
declared inside.

```spectec
def $enter_t(TC)
  = TC[ .LOCAL.FRAMES = $empty_typeFrame :: TC.LOCAL.FRAMES ]

def $exit_t(TC) = TC[ .LOCAL.FRAMES = typeFrame_t* ]
  -- if typeFrame_h :: typeFrame_t* = TC.LOCAL.FRAMES
```

`$enter_t` prepends an empty frame with `::`. `$exit_t` discards the head frame
by pattern-matching the stack as `typeFrame_h :: typeFrame_t*` and
reconstructing the context with only the tail.

This pair is used together around a block body, ensuring that variables declared
inside a block cannot escape it.

## Adders

The adder functions insert a new binding into the appropriate layer. Each one
first checks that the name is not already bound (preventing redeclaration), then
adds the new entry.

```spectec
def $add_var_t(LOCAL, TC, id, varTypeIR) = TC'
  -- if typeFrame_h :: typeFrame_t* = TC.LOCAL.FRAMES
  -- if ~$in_set<id>($dom_map<id, varTypeIR>(typeFrame_h), id)
  -- if typeFrame_h' = $add_map<id, varTypeIR>(typeFrame_h, id, varTypeIR)
  -- if TC' = TC[ .LOCAL.FRAMES = typeFrame_h' :: typeFrame_t* ]
```

The local adder only touches the _head_ frame, pointing to the innermost scope.
This is intentional: a variable declared in a nested block should not be visible
in the enclosing block.

There are three adder families:

- `$add_var_t(scope, typingContext, id, varTypeIR)`: adds a variable to a frame
- `$add_callableDef_t(typingContext, callableId, callableTypeDef)`: adds an
  action, parser, or control to the callable env
- `$add_typeDef_t(typingContext, typeId, typeDefIR)`: adds a struct, header,
  extern, or other type to the type def env

The latter two always write to the global layer, so they take no `scope`
argument.

## Finders

The finder functions look up a name and return its associated type. Variable
lookup follows a _scope chain_: check the local stack first, then fall through
to the block frame, then to the global frame.

```spectec
def $find_var_t(LOCAL, TC, id) = varTypeIR
  -- if typeFrame* = TC.LOCAL.FRAMES
  -- if varTypeIR = $find_maps<id, varTypeIR>(typeFrame*, id)

def $find_var_t(LOCAL, TC, id) = $find_var_t(BLOCK, TC, id)
  -- if typeFrame* = TC.LOCAL.FRAMES
  -- if eps = $find_maps<id, varTypeIR>(typeFrame*, id)
```

The two clauses for `LOCAL` form a conditional: the first succeeds if the
variable is found in the local frames; the second fires only when the first
returns `eps` (not found), and delegates to `BLOCK`. The `BLOCK` finder applies
the same pattern to fall through to `GLOBAL` if the variable is not in the block
frame.

This chain means a local variable can shadow a block parameter, and a block
parameter can shadow a global variable.

## The `TC` Meta-variable

After the syntax definitions, `5.00` also declares:

```spectec
var TC : typingContext
```

`TC` is shorthand for the typing context that gets threaded through nearly every
rule in the static semantics. Rather than writing `typingContext` in full each
time, the spec declares `TC` once as a _typed meta-variable_ of type
`typingContext`. The elaborator then recognizes `TC`, `TC'`, `TC_1`, and any
other suffix variant as standing for a value of that type, wherever they appear
in rule bodies or function definitions across all spec files.

## Exercise

**Branch:**
[`exercise/3.1`](https://github.com/pacokwon/nano-p4-spec/tree/exercise/3.1)

Check out the exercise branch in the `spec` submodule:

```shell
git -C nano-p4/spec checkout exercise/3.1
```

Run the following test to observe a failure:

```shell
./nano-p4spectec check nano-p4/spec/*.watsup -i nano-p4/include -p nano-p4/testdata/exercise/3.1.p4
```

The test _should_ pass (it is a valid program), but it does not. The
`Lvalue_ok/referenceExpression` rule attempts to find the block variable `pass`
but the call to `$find_var_t` fails.

```spectec
;; 5.04-typing-lvalue.watsup

rule Lvalue_ok/referenceExpression:
  scope TC |- referenceExpression : typeIR
  -- if id = $id(referenceExpression)
  -- if direction typeIR = $find_var_t(scope, TC, id) ;; <- fails!
  -- if direction = OUT \/ direction = INOUT
```

Add the missing clause to `$find_var_t` in `5.00-typing-context.watsup`.

When you are done, restore the original branch:

```shell
git -C nano-p4/spec checkout main
```
