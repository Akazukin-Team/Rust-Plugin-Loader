use crate::generate_abi;
use std::error::Error;
use std::sync::Arc;

generate_abi!(PluginVTable, {
    fn process(buffer: *mut f32, len: usize) -> i32;
    fn process2(value: f32);
});

pub struct PluginPort {
    // Arcで、ライブラリのアンロードを防ぎつつ高速アクセス
    vtable: Arc<PluginVTable>,
}

impl PluginPort {
    pub fn new(vtable: Arc<PluginVTable>) -> Self {
        let res = Self { vtable };
        res.check_vtable();
        res
    }

    fn check_vtable(&self) {
        if self.vtable.process.is_none() {
            panic!("process function is not found in library.");
        }
    }

    pub fn process(&self, buffer: &mut [f32]) {
        if let Some(func) = (*self.vtable).process {
            unsafe {
                func(buffer.as_mut_ptr(), buffer.len());
            }
        }
    }

    pub fn process2(&self, value: f32) {
        if let Some(func) = (*self.vtable).process2 {
            unsafe {
                func(value);
            }
        }
    }
}

pub struct PluginInstance {
    port: PluginPort,
}

impl PluginInstance {
    fn new(port: PluginPort) -> Self {
        Self { port }
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        // Load and cache a dynamic library
        let vtable = unsafe { PluginVTable::load(path)? };

        // Generate a port to refer vtable
        Ok(PluginInstance::new(PluginPort::new(vtable)))
    }

    pub fn get_port(&self) -> &PluginPort {
        &self.port
    }
}
