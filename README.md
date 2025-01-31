# clap-clap

[![CI](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml/badge.svg)](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml)

A [CLAP] plugin runtime. Very much WIP. 🚧

## Goals

* Provide a dynamical runtime environment to access safe-Rust [CLAP API].
* Follow CLAP terminology and the framework of CLAP extension modules.
* Build a testing and debugging platform for plugin authors.

[CLAP]: https://cleveraudio.org

[CLAP API]: https://github.com/free-audio/clap/tree/main/include/clap

## Example (ping-pong delay)

You can find an example how to implement a simple ping-pong delay [here].

To compile the source code, install Rust `>=1.85.0` (for the 2024 edition,
available on *nightly* and *beta* channels) and clone this repository together
with its submodules:

```bash
git clone --recurse-submodules https://github.com/mira-merkell/clap-clap
```

Build the example plugin with:

```bash
cargo build --example ping-pong --release
```

and look for the compiled dynamical library in `target/release/examples/`. The
name of the library is OS-specific:

* Linux: `libping_pong.so`
* Windows: `ping_pong.dll`
* macOS: `libping_pong.dylib`

Copy the file to where your DAW can find it and rename it to: `ping_pong.clap`.

*Note.*
If you're having problems trying to compile the sources on Windows,
remember to [install clang+llvm] first. If you're experiencing issues on macOS
due to bindgen's failing to generate valid declarations for `clap-sys`, see the
[workaround][GHA-workaround] I used for GHA, and the
ready-made [bindings][macos-bindings] for macOS/M1.

[here]: examples/ping_pong.rs

[install clang+llvm]: https://github.com/llvm/llvm-project/releases

[GHA-workaround]: ./.github/workflows/CI-darwin.yml

[macos-bindings]: ./.github/assets/bindings_darwin-M1_clap123.rs
