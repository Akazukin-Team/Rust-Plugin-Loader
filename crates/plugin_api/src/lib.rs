//! Plugin API: shared FFI types and small helpers for host/plugin interaction.
use std::os::raw::{c_char, c_void};

pub type PluginNameFn = unsafe extern "C" fn() -> *const c_char;
pub type PluginProcessFn = unsafe extern "C" fn(*const u8, usize) -> *const c_char;
pub type PluginInitFn = unsafe extern "C" fn() -> i32;
pub type PluginShutdownFn = unsafe extern "C" fn();
pub type CreateRendererFn = unsafe extern "C" fn() -> *mut c_void;
pub type DestroyRendererFn = unsafe extern "C" fn(*mut c_void);
pub type PluginOnLoadFn = unsafe extern "C" fn();
pub type PluginOnUnloadFn = unsafe extern "C" fn();
pub type PluginOnEnableFn = unsafe extern "C" fn();
pub type PluginOnDisableFn = unsafe extern "C" fn();
pub type PluginGetDependenciesFn = unsafe extern "C" fn() -> *const *const c_char;
pub type PluginVersionFn = unsafe extern "C" fn() -> *const c_char;
pub type PluginInitMetaFn = unsafe extern "C" fn() -> *const *const c_char;
pub type PluginFreeMetaFn = unsafe extern "C" fn();

// Re-export commonly-used std types for convenience
pub use std::ffi::CStr;
pub use std::ffi::CString;
pub use std::os::raw::c_char as CCHAR;
pub use std::os::raw::c_void as CVOID;
