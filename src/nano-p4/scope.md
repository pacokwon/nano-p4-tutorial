# Scope of Nano-P4

This section describes what Nano-P4 includes and what it leaves out. For a
reference of Nano-P4's grammar, consult
[Appendix A: Nano-P4 Grammar](./grammar.md)

## What Nano-P4 includes

### Types

Nano-P4 supports the following types:

- **Boolean** : `bool`
- **Fixed-width integers** : `bit<N>` and `int<N>` where `N` is a compile-time
  constant
- **`match_kind`** : the built-in kind used to annotate table key fields
- **Struct types** : declared with `struct`
- **Header types** : declared with `header`

Named types (structs and headers) can be used anywhere a type is expected.

### Expressions

Nano-P4 supports a standard set of expressions:

- Boolean and integer literals
- Variable references
- Unary operators: `!`, `~`, `-`, `+`
- Binary operators: arithmetic (`*`, `+`, `-`), comparison (`<`, `<=`, `>`,
  `>=`, `==`, `!=`), and bitwise/logical (`&`, `|`, `^`, `&&`, `||`)
- Member access: `expr.field`
- Function and extern calls: `f(args)`
- Parenthesized expressions

### Statements

Inside parser states, control apply blocks, and action bodies, the following
statements are available:

- Empty statement (`;`)
- Variable declaration with mandatory initializer: `type name = expr;`
- Assignment: `lvalue = expr;`
- Call statement: `lvalue(args);`
- Block: `{ ... }`
- Conditional: `if (expr) { ... } else { ... }` : both branches are required

### Actions

Top-level action declarations are supported:

```p4
action drop(inout Header hdr) {
    hdr.nanonet.drop = true;
}
```

Actions must be declared at the top level of the program. Nested action
declarations (inside a control block) are not supported.

### Tables

Tables are supported in a limited but usable form. A table must have a `key`
property and an `actions` property, and may optionally have a `const entries`
property:

```p4
table t {
    key = { hdr.nanonet.drop : exact; }
    actions = { drop(hdr); }
    const entries = {
        (true) : drop(hdr);
    }
}
```

Restrictions compared to full P4:

- Exactly one key field (the key block takes a single `expr : match_kind` entry)
- Control plane operations are not supported. That means runtime extension of
  table entries is not possible.
- No `default_action`, `size`, or other table properties
- Tables can only be declared inside a control block as a local declaration, not
  at the top level

### Parser block

Parser declarations are fully supported, including multiple named states and
`select` expressions for branching:

```p4
parser MyParser(packet_in pkt, out Header hdr) {
    state start {
        pkt.extract(hdr.nanonet);
        transition select(hdr.nanonet.drop) {
            true : drop_state;
            false : accept;
        }
    }
    state drop_state {
        transition accept;
    }
}
```

A parser state body consists of zero or more variable declarations followed by a
`transition` statement.

### Control block

Control declarations are supported. A control may contain local variable
declarations and table declarations, followed by an `apply` block:

```p4
control MyControl(inout Header hdr, out bool pass) {
    table t { ... }
    apply {
        t.apply();
    }
}
```

### Extern declarations

Extern object types (used to declare things like `packet_in`) can be declared
with method prototypes, but _not_ constructors:

```p4
extern packet_in {
    void extract(out Header hdr);
}
```

This is how the architecture model exposes built-in operations to the program.

### Instantiation

Top-level instantiation is supported and is how the main package is assembled:

```p4
NanoSwitch(MyParser(), MyControl()) main;
```

---

## What Nano-P4 excludes

The following P4 features are intentionally absent from Nano-P4.

### Types

- Header stacks (`header[N]`)
- `enum` types
- `header_union`
- `list` and `tuple` types
- Arbitrary-width integer literals

### Statements and control flow

- `for` loops
- `switch` statement
- `if` without `else`

### Declarations and scoping

- Type aliases (`typedef`, `type`)
- Constructor parameters on parsers/controls
- Nested action declarations

### Expressions and operators

- Implicit type casting
- Explicit type casting (`(T) expr`)
- Method overloading
- Dot-prefix notation (`.field` without a receiver)

### Table features

- `table.apply()` return object (`.hit`, `.action_run`)
- `default_action`
- `size` and other table properties
- Multiple key fields

### Header built-in methods

- `isValid()`, `setValid()`, `setInvalid()`, and similar header methods
