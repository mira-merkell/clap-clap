# clap-clap

[![CI](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml/badge.svg)](https://github.com/mira-merkell/clap-clap/actions/workflows/CI.yml)

A [CLAP] plugin runtime.  Documentation is available at [docs.rs][documentation].

[CLAP]: https://cleveraudio.org

[documentation]: https://docs.rs/clap-clap/latest/clap_clap/

## Goals

* Provide a dynamical runtime environment to access [CLAP API] from safe Rust.
* Follow closely CLAP module structure and terminology.
* Build a plugin testbed and a debugging platform.

This library is still very much WIP, 🚧 but the core functionality has already been implemented:
that includes all event types, audio and note ports, parameters, saving/loading plugin state,
logging, and reporting latency.  

New minor versions, `0.x.0` will be published once a month throughout 2025.
The first stable version: `1.0.0-beta` is planned for Sep 2025.

[CLAP API]: https://github.com/free-audio/clap/tree/main/include/clap

[extensions]: https://docs.rs/clap-clap/latest/clap_clap/ext/index.html

[`audio_ports`]: https://docs.rs/clap-clap/latest/clap_clap/ext/audio_ports/index.html

[`latency`]: https://docs.rs/clap-clap/latest/clap_clap/ext/latency/index.html

[`log`]: https://docs.rs/clap-clap/latest/clap_clap/ext/log/index.html

[`params`]:https://docs.rs/clap-clap/latest/clap_clap/ext/params/index.html

## Example (plugin template)

You can find the source code of a simple plugin template in:
[`./examples/plugin_template.rs`].

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

[`./examples/plugin_template.rs`]: ./examples/plugin_template.rs

## Installation

Simply add this library as a dependency of your crate:

```bash
cargo add clap-clap
```

A CLAP plugin is a dynamical library with C ABI and a symbol: `clap_entry`
visible to the host to load the plugin. To compile your plugin with the
right ABI, specify the crate type in your `Cargo.toml`:

```toml
# Your crate's Cargo.toml:
[lib]
crate-type = ["cdylib"]
```

To export the entry symbols, use the provided [`clap_clap::entry!`] macro. The
macro must be invoked exactly once in the entire plugin code, but you can
specify multiple types as arguments. For example, assuming  `MyPlug` and
`MyPlugToo` implement the trait [`Plugin`], you can export them with:

```rust
// Your crate's lib.rs:
clap_clap::entry!(MyPlug, MyPlugToo);
```

This will also build a plugin factory that a CLAP host can use to crate
instances of your plugins. The bundle will be a one compiled artefact that you
can install as a `*.clap` file.

[`clap_clap::entry!`]: https://docs.rs/clap-clap/latest/clap_clap/macro.entry.html

[`Plugin`]: https://docs.rs/clap-clap/latest/clap_clap/plugin/trait.Plugin.html

## Contributing

All contributions are welcome!

Help with writing documentation and examples will be much needed in the summer.
Alternatively, if you could set aside a few hours in late 2025 to help with
testing sample plugins before the stable release, that would be greatly
appreciated. 🎈

## Credits and License

Copyright (c) 2025 ⧉⧉⧉

This software is distributed under the MIT License. See [LICENSE](./LICENSE)
for more information.

Online repository available at: https://github.com/mira-merkell/clap-clap

### Maintainers

* ⧉⧉⧉
* Marek Miller <mlm@math.ku.dk>

Report bugs or submit patches via [GitHub Issues].

[GitHub Issues]: https://github.com/mira-merkell/clap-clap/issues
