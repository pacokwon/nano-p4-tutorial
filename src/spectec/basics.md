# Basics

P4-SpecTec specifications are written in `.watsup` files using a custom DSL.
This section walks through the core constructs of that DSL using examples from
the [SpecTecX tutorial](https://github.com/kaist-plrg/spectecx/releases/tag/tutorial-rc6),
which specifies *Typed Imp*, a small typed imperative language.

## Comments

Single-line comments start with `;;`. Additional semicolons (`;;;`, `;;;;`, ...)
are used by convention to indicate heading levels in the document structure,
but have no semantic difference.

```spectec
;;--------------------
;; Syntax

;;--- Contexts ---
```

## Primitive Data Types

P4-SpecTec has the following primitive types:

| Type   | Description                        |
|--------|------------------------------------|
| `bool` | Boolean (`true` or `false`)        |
| `int`  | Arbitrary-precision integer        |
| `nat`  | Non-negative integer               |
| `text` | String                             |

These are the types used in meta-variable declarations and function signatures.
They are distinct from the types of the target language being specified.
For example, `INT` in the Typed Imp spec is a syntax constructor defined with
`syntax`, not the primitive `int` of the DSL itself.

## Options and Lists

Option types are written by appending `?` to a type. The absent case is `eps`,
and a present value is written directly:

```spectec
dec $lookup<K, V>(map<K, V>, K) : V?

def $lookup<K, V>(eps, K_query) = eps    ;; absent: key not found
def $lookup<K, V>((K_h -> V_h)::_, K_query) = V_h
  -- if K_h = K_query                    ;; present: return the value
```

An `if` premise can match on an option result:

```spectec
rule Check_expr/id:
  tenv |- x : t
  -- if $lookup<id, type>(tenv, x) = t   ;; binds t if present, fails if eps
```

List types are written by appending `*` to a type. The empty list is `eps`,
and `::` is the cons operator:

```spectec
syntax map<K,V> = (pair<K,V>)*
```

Pattern matching on lists follows the same `::` notation in `def` cases:

```spectec
def $lookup<K, V>(eps, K_query) = eps
def $lookup<K, V>((K_h -> V_h)::(K_t -> V_t)*, K_query) = V_h
  -- if K_h = K_query
```

Lists also appear in rule conclusions and premises to prepend an entry to a context:

```spectec
rule Check_command/decl:
  tenv |- (t x `= e) -| (x -> t)::tenv
  -- Check_expr: tenv |- e : t
```

Here `(x -> t)::tenv` constructs a new context with the binding `x -> t`
prepended to the existing context `tenv`.

## Syntax Definitions

The `syntax` keyword defines a grammar production, which is the abstract syntax of
the language being specified.

```spectec
syntax type =
  | INT
  | BOOL
  | type `-> type

syntax literal =
  | `NUM int
  | `BOOL bool

syntax expr =
  | literal
  | id
  | `! expr
  | expr `+ expr
  | expr `<= expr
```

Each alternative is prefixed with `|`. Names starting with a lowercase letter
refer to built-in types or *non-terminals*, which are built-in or previously
declared syntax types.
Uppercase names (like `INT`, `BOOL`) and backtick-prefixed tokens
(like `` `NUM ``, `` `-> ``, `` `+ ``) are *terminals*.

## Meta-variables

The `var` keyword declares metavariables and their types. These act as
shorthands: wherever `e` appears unbound in a rule, it is implicitly typed as
`expr`.

```spectec
var i : int
var b : bool
var x : id
var e : expr
var c : command
var t : type
```

A variable's type is inferred from its name by stripping a trailing suffix
that begins at the first `_` or `'` character. For example, given `var e : expr`,
the following names are all valid `expr` variables:

```spectec
rule Check_expr/add:
  tenv |- e_l `+ e_r : INT   ;; e_l, e_r -> strip at '_' -> e : expr
  -- Check_expr: tenv |- e_l : INT
  -- Check_expr: tenv |- e_r : INT

rule Check_expr/not:
  tenv |- `! e' : BOOL        ;; e' -> strip at '\'' -> e : expr
  -- Check_expr: tenv |- e' : BOOL
```

A suffix of only underscores (`e__`) or a purely alphanumeric suffix without
a separator (`eLeft`) does not strip, so those would not be recognized as
`expr` variables and would cause an error:

```spectec
rule Check_expr/not:
  tenv |- `! eLeft : BOOL     ;; ERROR: eLeft does not resolve to a known variable
  -- Check_expr: tenv |- eLeft : BOOL
```

## Function Declarations and Definitions

Functions are declared with `dec` and defined with `def`. The declaration
gives the name, argument types, and return type. Definitions provide
pattern-matched cases. Each of those cases are called *clause*s.

```spectec
dec $lookup<K, V>(map<K, V>, K) : V?

;; If map is empty, return empty
def $lookup<K, V>(eps, K_query) = eps

;; If head entry's key matches query, return its value
def $lookup<K, V>((K_h -> V_h)::(K_t -> V_t)*, K_query) = V_h
  -- if K_h = K_query

;; If head entry's key does not match query, recursively call on the rest
def $lookup<K, V>((K_h -> V_h)::(K_t -> V_t)*, K_query)
  = $lookup<K, V>((K_t -> V_t)*, K_query)
  -- otherwise
```

A few things to note:

- Function names are prefixed with `$`.
- Angle brackets introduce type parameters (e.g. `<K, V>`). They must be made
  explicit in function calls.
- The return type `V?` means an optional value (`eps` represents the absent case).
- Each `def` case can have side conditions introduced with `if`, or a
  catch-all `otherwise`.
- On function call, each clause is evaluated from top to bottom. If pattern match
  fails or `if` premises are not satisfied, the clause *fails* and tries the next
  clause.

The `builtin` modifier marks functions whose implementation is provided by
the toolchain rather than defined in the spec:

```spectec
builtin dec $sum_nat(nat*) : nat
```

## Relations and Rules

A *relation* defines the signature for a set of *rules*.

```spectec
;; Typecheck `expr` under context `tenv`
relation Check_expr:
  tenv |- expr : type
  hint(input %0 %1)
  hint(prose_in "typechecking" %1 "under context" %0)
```

This declares a judgment `tenv |- expr : type`, meaning "expression `expr`
has type `type` under typing context `tenv`."
The `|-` symbol (turnstile) is conventional notation borrowed from type theory.
In P4-SpecTec, it is just a separator between the context and the subject.

`%0`, `%1`, etc. refer to the positional components of the judgment.
`hint(input ...)` specifies which components are inputs to the relation.
Here, `%0` (`tenv`) and `%1` (`expr`) are inputs, and `%2` (`type`) is the output.

*Rules* define *when* a relation holds.
Each rule has a *conclusion* (the judgment being established) and zero or
more *premises* (the conditions that must hold), introduced with `--`:

```spectec
;; If expression is integer literal, it has type INT
rule Check_expr/num:
  tenv |- (`NUM i) : INT

;; If expression is logical not,
;;   Check if the operand is BOOL, then it has type BOOL
rule Check_expr/not:
  tenv |- `! e : BOOL
  -- Check_expr: tenv |- e : BOOL

;; If expression is binary addition,
;;   Check if both operands are INT, then it has type INT
rule Check_expr/add:
  tenv |- e_l `+ e_r : INT
  -- Check_expr: tenv |- e_l : INT
  -- Check_expr: tenv |- e_r : INT
```

The first rule has no premises; integer literals always have type `INT`.
The second and third rules invoke `Check_expr` recursively as premises.

Unlike traditional declarative inference rules where premises are unordered
and existential witnesses may be guessed non-deterministically, P4-SpecTec
rules are *algorithmic*: premises are executed in order, from top to bottom,
and every value must be computed from already-known inputs.
This makes rules directly executable as a type checker or interpreter.

An `if` premise introduces a side condition that does not invoke another
relation:

```spectec
rule Check_expr/id:
  tenv |- x : t
  -- if $lookup<id, type>(tenv, x) = t
```

The `=` in an `if` premise is overloaded: if the right-hand side is already
known, it is a *check*; if it is an unbound meta-variable, it becomes a
*binding* that computes the value from the left-hand side.
Here, `t` is unbound, so `$lookup` is called and its result is bound to `t`,
which is then used in the conclusion `tenv |- x : t`.
If `$lookup` returns `eps` (absent), the rule *fails* and the next rule is tried.

Rules can mix `if` and relation premises freely:

```spectec
rule Check_command/assign:
  tenv |- (x `= e) -| tenv
  -- Check_expr: tenv |- e : t
  -- if $lookup<id, type>(tenv, x) = t
```

Here the relation premise `Check_expr` runs first and binds `t`, then the
`if` premise checks that `x` is already declared with that same type.

## Iteration

The `(pattern)*` syntax applies a pattern element-wise over a list.

For example, the following clause (from the nano-p4 spec) extracts the third
component of every element in `parameterIR*` into a new list `nameIR*`:

```spectec
def $distinct_params(parameterIR*)
  = $distinct_<nameIR>(nameIR*)
  -- if (_ _ nameIR = parameterIR)*
```

The `if` premise runs the pattern `_ _ nameIR = parameterIR` once per element,
binding `nameIR` at each position and collecting the results into `nameIR*`.

More advanced usages of iteration are to be demonstrated further into the tutorial.

## Hints

Hints are metadata annotations that guide the *prose backend* when generating
human-readable documentation.

```spectec
relation Eval_expr:
  env |- expr ==> value
  hint(input %0 %1)
  hint(prose_in "evaluating" %1 "in environment" %0)
```

`hint(prose_in ...)` controls the generated prose description.

Individual rules can also carry hints:

```spectec
syntax literal =
  | `NUM int   hint(prose "the integer" %0)
  | `BOOL bool hint(prose "the boolean" %0)
```

We will cover the prose backend in more detail later in this tutorial.

## Exercise

If you're interested in a more hands-on introduction, we strongly
recommend you to go and complete the [SpecTecX tutorial](https://github.com/kaist-plrg/spectecx/releases/tag/tutorial-rc6).
