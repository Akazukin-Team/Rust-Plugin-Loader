use crate::generate_abi;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use std::sync::Arc;

// Generate a vtable for plugin symbols. Names here must match exported symbol names.
generate_abi!(PluginVTable, {
    fn plugin_get_name() -> *const c_char;
    fn plugin_get_version() -> *const c_char;
    fn plugin_get_dependencies() -> *mut *const c_char;
    fn plugin_init();
    fn plugin_free();
    fn plugin_on_load();
    fn plugin_on_unload();
    fn plugin_on_enable();
    fn plugin_on_disable();
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
    pub meta: PluginMeta,
    pub vtable: Option<Arc<PluginVTable>>,
}

impl PluginContext {
    pub fn new(meta: PluginMeta, vtable: Option<Arc<PluginVTable>>) -> Self {
        Self { meta, vtable }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let vtable = PluginVTable::load(path.as_ref().to_str().ok_or("invalid path")?)
            .map_err(|e| e.to_string())?;

        // initialize meta by calling plugin_init if present
        let mut name = String::new();
        let mut version = String::new();
        let mut deps_vec: Vec<String> = Vec::new();
        let vt_ref: &PluginVTable = &vtable;
        // read name/version from dedicated getters (if present)

        if let Some(get_name) = vt_ref.plugin_get_name() {
            unsafe {
                let p = get_name();
                if !p.is_null() {
                    name = CStr::from_ptr(p).to_string_lossy().into_owned();
                }
            }
        }
        if let Some(get_ver) = vt_ref.plugin_get_version() {
            unsafe {
                let p = get_ver();
                if !p.is_null() {
                    version = CStr::from_ptr(p).to_string_lossy().into_owned();
                }
            }
        }
        if let Some(get_deps) = vt_ref.plugin_get_dependencies() {
            unsafe {
                let arr = get_deps();
                if !arr.is_null() {
                    let mut idx = 0usize;
                    loop {
                        let p = *arr.add(idx);
                        if p.is_null() {
                            break;
                        }
                        deps_vec.push(CStr::from_ptr(p).to_string_lossy().into_owned());
                        idx += 1;
                    }
                }
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
    pub address: *const c_int,
    pub len: usize;
}

impl Array {
    pub fn to_element(vec: Vec<Any>) -> Array {
        Array {
            address:vec.as_ptr()
    }
}

pub extern "C" get_arr() -> Array {
}
