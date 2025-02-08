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

To compile the sources, install Rust `1.85.0` or later (for the 2024
edition, available on *nightly* and *beta* channels) and clone the repository:

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

# Compatibility

This library aims at full compatibility with the CLAP API. A lot of
functionality, however, is still missing. As a workaround, the library's [
`Host`] type provides two methods that you can use to obtain raw pointers to
CLAP's structs [ `clap_host`] and [`clap_plugin`]. During the initialization of
the plugin, use the host handle like this:

```rust
impl Plugin for MyPlug {
    fn init(&mut self, host: Arc<clap::Host>) -> Result<(), clap_clap::Error> {
        let clap_host = unsafe { host._raw_clap_host() };
        let clap_plugin = unsafe { host._raw_clap_plugin() };

        // (...)

        Ok(())
    }
}
```

You can then take the pointer to the raw `clap_plugin` to, e.g., wrap
`clap_plugin.get_extension()` in your own function providing additional
functionality. Note that the pointer: `clap_plugin.plugin_data` is reserved by
this library's runtime, and overriding it will most surely result in undefined
behavior.

Since it would be difficult to define safety requirements for those methods,
they should be considered a last resort, generally unsafe, and their use is
discouraged.

See also the module: [`clap_clap::ffi`] for the definitions of raw CLAP
structures.


[`Host`]: https://docs.rs/clap-clap/latest/clap_clap/host/struct.Host.html

[`clap_host`]: https://github.com/free-audio/clap/blob/main/include/clap/host.h

[`clap_plugin`]: https://github.com/free-audio/clap/blob/main/include/clap/plugin.h

[`clap_clap::ffi`]: https://docs.rs/clap-clap/latest/clap_clap/ffi/index.html