//use crate::generate_abi;
use macros::{generate_ffi_object, generate_vtable};
use std::any::Any;
use std::ffi::{CStr, c_int};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::Arc;

generate_ffi_object!(RaiiString, *const std::os::raw::c_char, plugin_free_string);

generate_vtable!(PluginVTable, {
    fn plugin_get_name() -> (*const c_char -> RaiiString);
    fn plugin_get_version() -> (*const c_char -> RaiiString);
    optional fn plugin_get_dependencies() -> (*const c_char -> RaiiString);
    optional fn plugin_init();
    optional fn plugin_free();
    optional fn plugin_on_load();
    optional fn plugin_on_unload();
    optional fn plugin_on_enable();
    optional fn plugin_on_disable();
});

pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub deps: Vec<String>,
}

impl PluginMeta {
    pub fn new(name: String, version: String, deps: Vec<String>) -> Self {
        Self {
            name,
            version,
            deps,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_version(&self) -> &String {
        &self.version
    }

    pub fn get_dependencies(&self) -> &Vec<String> {
        &self.deps
    }
}

pub struct PluginContext {
    meta: PluginMeta,
    vtable: Option<Arc<PluginVTable>>,
}

impl PluginContext {
    pub fn new(meta: PluginMeta, vtable: Option<Arc<PluginVTable>>) -> Self {
        Self { meta, vtable }
    }

    pub fn get_meta(&self) -> &PluginMeta {
        &self.meta
    }

    pub fn get_vtable(&self) -> Option<Arc<PluginVTable>> {
        self.vtable.clone()
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let vtable = PluginVTable::load(path.as_ref().to_str().ok_or("invalid path")?)
            .map_err(|e| e.to_string())?;

        // initialize meta by calling plugin_init if present
        let name: String;
        let version: String;
        let mut deps_vec: Vec<String> = Vec::new();
        let vt_ref: &PluginVTable = &vtable;

        {
            let plugin_name = vt_ref.plugin_get_name()();
            unsafe {
                name = CStr::from_ptr(*plugin_name.get())
                    .to_string_lossy()
                    .into_owned();
            }
        }
        {
            let plugin_version = vt_ref.plugin_get_version()();
            unsafe {
                version = CStr::from_ptr(*plugin_version.get())
                    .to_string_lossy()
                    .into_owned();
            }
        }
        if let Some(get_deps) = vt_ref.plugin_get_dependencies() {
            unsafe {
                let arr = get_deps();
            }
        }

        if name.is_empty() {
            return Err("plugin did not provide a name".to_string());
        }
        if version.is_empty() {
            return Err("plugin did not provide a version".to_string());
        }

        let meta = PluginMeta::new(name, version, deps_vec);
        Ok(PluginContext::new(meta, Some(vtable)))
    }
}

#[repr(C)]
pub struct Array {
    pub addr: *const c_int,
    pub len: usize,
}

generate_ffi_object!(RaiiArray, Array, plugin_free_array);
