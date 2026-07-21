# P4-SpecTec

[P4-SpecTec](https://github.com/kaist-plrg/p4-spectec) is a mechanization toolchain for the P4 programming language.
It provides a domain-specific language for writing formal specifications in the form of *algorithmic inference rules*.

By writing typing rules with P4-SpecTec, you get a reference type checker.
By writing dynamic semantics rules, you get a reference interpreter.
The *prose backend* also allows one to generate a human-readable documentation
from a specification written in P4-SpecTec.

In this section, we will cover the following:

- **Installation**: how to build and install P4-SpecTec
- **Basics**: the core syntax and constructs of P4-SpecTec
- **Workflow**: how to run the toolchain and interpret its output
- **Standard Library**: the built-in utility functions used throughout the spec
