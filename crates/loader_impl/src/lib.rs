/*
mod manager;
mod plugin;
mod plugin2;

pub use manager::{PluginManager, get_global_plugin_manager, set_global_plugin_manager};
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
*/
/*
use macros::{generate_ffi_object, generate_vtable};


generate_ffi_object!(RaiiString, *const std::os::raw::c_char, plugin_free_string);

generate_vtable!(PluginVTable, {
    PluginGetName: fn plugin_get_name() -> (*const std::os::raw::c_char, RaiiString);
    PluginGetDisplayName: optional fn plugin_get_displayname() -> (*const std::os::raw::c_char, RaiiString);
    PluginInit: fn plugin_init();
    PluginOnLoaded: optional fn plugin_on_loaded();
});

struct A;
impl A {
    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let vtable = PluginVTable::load("my_plugin.dll")?;

        if vtable.plugin_get_name.exists() {
            let ptr_ref = vtable.plugin_get_name.call();
            unsafe {
                let c_str = std::ffi::CStr::from_ptr(*ptr_ref.get());
                println!("Plugin Name: {}", c_str.to_string_lossy());
            }
        }
        Ok(())
    }
}*/
pub mod api;
pub mod extension;
pub mod manager;
pub mod plugin;

pub mod plugin3 {
    use plugin_api::VTableError;
    use std::ffi::{CStr, c_int};
    use std::os::raw::c_char;
    use std::path::Path;
    use std::sync::Arc;
    pub struct RaiiString {
        ptr: Option<*const std::os::raw::c_char>,
        free_fn: unsafe extern "C" fn(*const std::os::raw::c_char),
        lib: std::sync::Arc<libloading::Library>,
    }
    impl RaiiString {
        pub fn new(
            ptr: *const std::os::raw::c_char,
            lib: std::sync::Arc<libloading::Library>,
        ) -> Self {
            let free_fn: unsafe extern "C" fn(*const std::os::raw::c_char) = unsafe {
                let sym: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char)> =
                    lib.get("plugin_free_string\0".as_bytes()).unwrap();
                *sym
            };
            Self {
                ptr: Some(ptr),
                free_fn,
                lib,
            }
        }
        pub fn get(&self) -> &*const std::os::raw::c_char {
            self.ptr.as_ref().expect("The raw object is already taken")
        }
        pub fn into_inner(mut self) -> *const std::os::raw::c_char {
            let ptr = self.ptr.take().expect("The raw object is already taken");
            std::mem::forget(self);
            ptr
        }
    }
    impl Drop for RaiiString {
        fn drop(&mut self) {
            if let Some(ptr) = self.ptr.take() {
                unsafe {
                    (self.free_fn)(ptr);
                }
            }
        }
    }
    impl plugin_api::FfiWrapper for RaiiString {
        type Raw = *const std::os::raw::c_char;
        fn from_raw(raw: Self::Raw, lib: std::sync::Arc<libloading::Library>) -> Self {
            Self::new(raw, lib)
        }
    }
    pub struct PluginVTable {
        _lib: std::sync::Arc<libloading::Library>,
        pub plugin_get_name: unsafe extern "C" fn() -> *const c_char,
        pub plugin_get_version: unsafe extern "C" fn() -> *const c_char,
        pub plugin_get_dependencies: Option<unsafe extern "C" fn() -> *const c_char>,
        pub plugin_init: Option<unsafe extern "C" fn() -> ()>,
        pub plugin_free: Option<unsafe extern "C" fn() -> ()>,
        pub plugin_on_load: Option<unsafe extern "C" fn() -> ()>,
        pub plugin_on_unload: Option<unsafe extern "C" fn() -> ()>,
        pub plugin_on_enable: Option<unsafe extern "C" fn() -> ()>,
        pub plugin_on_disable: Option<unsafe extern "C" fn() -> ()>,
    }
    impl PluginVTable {
        pub fn load(path: &str) -> Result<std::sync::Arc<Self>, VTableError> {
            let lib = std::sync::Arc::new(unsafe {
                libloading::Library::new(path).map_err(|e| VTableError::LoadingError(e))?
            });
            Self::load_from_lib(lib)
        }
        pub fn load_from_lib(
            lib: std::sync::Arc<libloading::Library>,
        ) -> Result<std::sync::Arc<Self>, VTableError> {
            let raw_fn = unsafe { (&*lib).get("plugin_get_name\0".as_bytes()) }
                .ok()
                .map(|s| *s);
            let plugin_get_name =
                raw_fn.expect("Required function \'plugin_get_name\' not found in library");
            let plugin_get_version: unsafe extern "C" fn() -> *const c_char = {
                let res = unsafe { (&*lib).get("plugin_get_version\0".as_bytes()) };
                let ret: Option<unsafe extern "C" fn() -> *const c_char> = match res {
                    Ok(sym) => Ok(Some(*sym)),
                    Err(e) => match e {
                        libloading::Error::DlSym { .. }
                        | libloading::Error::DlSymUnknown { .. }
                        | libloading::Error::GetProcAddress { .. }
                        | libloading::Error::GetProcAddressUnknown { .. } => Ok(None),
                        other => Err(VTableError::LoadingError(other)),
                    },
                }?;

                let ret = ret.ok_or(VTableError::MissingFunction(
                    "plugin_get_version".to_string(),
                ))?;
                ret
            };

            let raw_fn = unsafe { (&*lib).get("plugin_get_dependencies\0".as_bytes()) }
                .ok()
                .map(|s| *s);
            let plugin_get_dependencies = raw_fn;
            let raw_fn = unsafe { (&*lib).get("plugin_init\0".as_bytes()) }
                .ok()
                .map(|s| *s);
            let plugin_init = raw_fn;
            let raw_fn = unsafe { (&*lib).get("plugin_free\0".as_bytes()) }
                .ok()
                .map(|s| *s);
            let plugin_free = raw_fn;
            let raw_fn = unsafe { (&*lib).get("plugin_on_load\0".as_bytes()) }
                .ok()
                .map(|s| *s);
            let plugin_on_load = raw_fn;
            let raw_fn = unsafe { (&*lib).get("plugin_on_unload\0".as_bytes()) }
                .ok()
                .map(|s| *s);
            let plugin_on_unload = raw_fn;
            let raw_fn = unsafe { (&*lib).get("plugin_on_enable\0".as_bytes()) }
                .ok()
                .map(|s| *s);
            let plugin_on_enable = raw_fn;
            let raw_fn = unsafe { (&*lib).get("plugin_on_disable\0".as_bytes()) }
                .ok()
                .map(|s| *s);
            let plugin_on_disable = raw_fn;
            let table = Self {
                _lib: std::sync::Arc::clone(&lib),
                plugin_get_name,
                plugin_get_version,
                plugin_get_dependencies,
                plugin_init,
                plugin_free,
                plugin_on_load,
                plugin_on_unload,
                plugin_on_enable,
                plugin_on_disable,
            };
            Ok(std::sync::Arc::new(table))
        }
        pub fn library(&self) -> std::sync::Arc<libloading::Library> {
            std::sync::Arc::clone(&self._lib)
        }
        pub fn plugin_get_name(&self) -> impl Fn() -> RaiiString + '_ {
            move || {
                let raw_ret = unsafe { (self.plugin_get_name)() };
                plugin_api::FfiWrapper::from_raw(raw_ret, self.library())
            }
        }
        pub fn plugin_get_version(&self) -> impl Fn() -> RaiiString + '_ {
            move || {
                let raw_ret = unsafe { (self.plugin_get_version)() };
                plugin_api::FfiWrapper::from_raw(raw_ret, self.library())
            }
        }
        pub fn plugin_get_dependencies(&self) -> Option<impl Fn() -> RaiiString + '_> {
            self.plugin_get_dependencies.as_ref().map(|func| {
                move || {
                    let raw_ret = unsafe { func() };
                    plugin_api::FfiWrapper::from_raw(raw_ret, self.library())
                }
            })
        }
        pub fn plugin_init(&self) -> Option<impl Fn() -> () + '_> {
            self.plugin_init
                .as_ref()
                .map(|func| move || unsafe { func() })
        }
        pub fn plugin_free(&self) -> Option<impl Fn() -> () + '_> {
            self.plugin_free
                .as_ref()
                .map(|func| move || unsafe { func() })
        }
        pub fn plugin_on_load(&self) -> Option<impl Fn() -> () + '_> {
            self.plugin_on_load
                .as_ref()
                .map(|func| move || unsafe { func() })
        }
        pub fn plugin_on_unload(&self) -> Option<impl Fn() -> () + '_> {
            self.plugin_on_unload
                .as_ref()
                .map(|func| move || unsafe { func() })
        }
        pub fn plugin_on_enable(&self) -> Option<impl Fn() -> () + '_> {
            self.plugin_on_enable
                .as_ref()
                .map(|func| move || unsafe { func() })
        }
        pub fn plugin_on_disable(&self) -> Option<impl Fn() -> () + '_> {
            self.plugin_on_disable
                .as_ref()
                .map(|func| move || unsafe { func() })
        }
    }
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
    pub struct RaiiArray {
        ptr: Option<Array>,
        free_fn: unsafe extern "C" fn(Array),
        lib: std::sync::Arc<libloading::Library>,
    }
    impl RaiiArray {
        pub fn new(ptr: Array, lib: std::sync::Arc<libloading::Library>) -> Self {
            let free_fn: unsafe extern "C" fn(Array) = unsafe {
                let sym: libloading::Symbol<unsafe extern "C" fn(Array)> =
                    lib.get("plugin_free_array\0".as_bytes()).unwrap();
                *sym
            };
            Self {
                ptr: Some(ptr),
                free_fn,
                lib,
            }
        }
        pub fn get(&self) -> &Array {
            self.ptr.as_ref().expect("The raw object is already taken")
        }
        pub fn into_inner(mut self) -> Array {
            let ptr = self.ptr.take().expect("The raw object is already taken");
            std::mem::forget(self);
            ptr
        }
    }
    impl Drop for RaiiArray {
        fn drop(&mut self) {
            if let Some(ptr) = self.ptr.take() {
                unsafe {
                    (self.free_fn)(ptr);
                }
            }
        }
    }
    impl plugin_api::FfiWrapper for RaiiArray {
        type Raw = Array;
        fn from_raw(raw: Self::Raw, lib: std::sync::Arc<libloading::Library>) -> Self {
            Self::new(raw, lib)
        }
    }
}
