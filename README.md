# clap-clap

[![CI](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml/badge.svg)](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml)

A [CLAP] plugin runtime. Very much WIP. 🚧

Documentation available
at [docs.rs](https://docs.rs/clap-clap/latest/clap_clap/).

## Goals

* Provide a dynamical runtime environment to access [CLAP API] from safe Rust.
* Follow CLAP terminology and the framework of CLAP extension modules.
* Build a testing and debugging platform for plugin authors.

[CLAP]: https://cleveraudio.org

[CLAP API]: https://github.com/free-audio/clap/tree/main/include/clap

## Example (plugin template)

You can find the source code of a simple plugin template [here].

To compile the sources, install Rust `1.85.0` or later (for the Rust 2024
edition) and clone the repository:

```bash
git clone https://github.com/mira-merkell/clap-clap
```

Build the example plugin with:

```bash
cargo build --example plugin_template --release
```

and look for the compiled dynamical library in `target/release/examples/`. The
name of the library is OS-specific:

* Linux: `libplugin_template.so`
* Windows: `plugin_template.dll`
* macOS: `libplugin_template.dylib`

Copy the file to where your DAW can find it and rename it to:
`plugin_template.clap`.

[here]: ./examples/plugin_template.rs