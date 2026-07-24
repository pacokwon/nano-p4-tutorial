# Writing Static Semantics Rules

Static semantics defines what it means for a Nano-P4 program to be _well-typed_.
Before a program runs, the type checker walks through every declaration,
statement, and expression to verify that types are used consistently and that
names refer to things that actually exist.

In this chapter, we read through the static semantics specification of Nano-P4
piece by piece. We will not build the spec from scratch. Instead, we will take a
functioning spec apart and understand what it means. But don't worry! There are
curated exercises for you at the end of every section, where you will debug or
extend a faulty version of the spec. Each section explains what a piece of the
spec says, why it is written that way, and how it connects to the P4-SpecTec
constructs you saw in the previous chapter.

At the end of this chapter, we will end up with a relation that type-checks an
entire program:

```spectec
relation Program_ok:
  |- program -| typingContext
  hint(input %0)
```

More specifically, we will cover the following topics:

- **Typing Context**: the data structures that hold type information as the
  checker walks the program
- **Types**: rules for validating type expressions and checking type equality
- **Expressions**: how the type of an expression is derived from its parts
- **Statements**: how statements are checked and how variable declarations
  extend the context
- **Parameters and Arguments**: how function signatures are elaborated and how
  call sites are validated against them
- **Declarations**: how top-level declarations such as actions, externs, type
  definitions, parsers, and controls are checked and registered
- **Tables**: how table keys and actions are validated
