use clap_clap::factory::{Factory, FactoryPluginPrototype};

use crate::shims::plugin::ShimPlugin;

#[path = "unit/shims.rs"]
mod shims;

// `factory.descriptor()` tried to access internal
// list with index out of bounds and panicked.
#[test]
fn factory_descriptor_index_out_of_bounds() {
    let factory = Factory::new(vec![Box::new(
        FactoryPluginPrototype::<ShimPlugin>::build().unwrap(),
    )]);

    let _ = factory.descriptor(1).unwrap_err();
}
