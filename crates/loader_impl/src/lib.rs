mod generate_abi;
mod manager;
mod plugin;
mod plugin2;

pub use manager::{PluginManager, get_global_plugin_manager, set_global_plugin_manager};
pub use plugin::Plugin;
pub use plugin2::PluginInstance;
use std::error::Error;

// Loader version exported so plugins can declare dependency on the loader itself.
pub const LOADER_VERSION: &str = "0.1.0";

pub fn loader_version() -> &'static str {
    LOADER_VERSION
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load the dynamic library
    let plugin = PluginInstance::load("./my_plugin.so")?;

    // Dummy data
    let mut vec = vec![];

    // Execute functions across the ABI boundary
    plugin.get_port().process(&mut vec);
    plugin.get_port().process2(0.8);

    Ok(())
}
