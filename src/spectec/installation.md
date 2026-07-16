# Installation

We will be using the `nano-p4spectec` binary and `make` throughout this tutorial.
This section demonstrates how to build the `nano-p4spectec` binary from the
P4-SpecTec source.

## Cloning the Repository

```shell
$ git clone https://github.com/kaist-plrg/p4-spectec.git
$ cd p4-spectec
```

## Building from Source

### Prerequisites

**Linux**

```shell
$ apt-get install opam
$ opam init
```

**macOS**

Install `opam` version 2.0.5 or higher following the instructions at [ocaml.org](https://ocaml.org/docs/installing-ocaml).

You may also need `libgmp-dev` and `pkg-config` depending on your system.

**NixOS**

If you use [Nix](https://nixos.org/), a `flake.nix` is provided that
sets up the full development environment automatically:

```shell
$ nix develop
$ make release
```

This drops you into a shell with OCaml 5.1 and all required packages available,
without needing to manage `opam` manually.

### OCaml Compiler and Packages

```shell
$ opam switch create 5.1.0
$ eval $(opam env)
$ opam install dune bignum 'menhir=20240715' 'menhirLib=20240715' core core_unix bisect_ppx yojson ppx_deriving_yojson
```

### Building

```shell
$ make release
```

This creates the `nano-p4spectec` executable in the project root.
