use std::{ffi::c_char, sync::Arc};

use macros::generate_vtable;

use crate::plugin::{Array, RaiiArray};

#[repr(C)]
pub struct ExtensionPoint {
    plugin_id: *const c_char,
    extension_id: *const c_char,
}

generate_vtable!(ExtensionVTable, {
    fn extension_get_points() -> (Array -> RaiiArray);
});

pub fn get_extension_points(lib: Arc<libloading::Library>) -> Vec<ExtensionPoint> {
    let vtable = ExtensionVTable::load_from_lib(lib);
    let raw_exts = vtable.extension_get_points();
    let exts = unsafe {
        std::slice::from_raw_parts();
    };
}
