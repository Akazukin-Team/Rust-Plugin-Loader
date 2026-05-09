use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Once;

static mut META_META_STRINGS: Option<Box<Vec<CString>>> = None;
static mut META_META_PTRS: Option<Box<[*const c_char]>> = None;
static INIT_META: Once = Once::new();

static mut META_DEPS_STRINGS: Option<Box<Vec<CString>>> = None;
static mut META_DEPS_PTRS: Option<Box<[*const c_char]>> = None;
static INIT_DEPS: Once = Once::new();

#[no_mangle]
pub extern "C" fn plugin_init() -> *const *const c_char {
    unsafe {
        INIT_META.call_once(|| {
            let mut vec: Vec<CString> = Vec::new();
            vec.push(CString::new("example_plugin").unwrap());
            vec.push(CString::new("0.1.0").unwrap());

            let mut ptrs: Vec<*const c_char> = Vec::with_capacity(vec.len() + 1);
            for s in &vec {
                ptrs.push(s.as_ptr());
            }
            ptrs.push(std::ptr::null());

            META_META_PTRS = Some(ptrs.into_boxed_slice());
            META_META_STRINGS = Some(Box::new(vec));
        });
        META_META_PTRS.as_ref().unwrap().as_ptr()
    }
}

#[no_mangle]
pub extern "C" fn plugin_free() {
    unsafe {
        if let Some(boxed) = META_META_STRINGS.take() {
            drop(boxed);
        }
        if let Some(boxed_ptrs) = META_META_PTRS.take() {
            drop(boxed_ptrs);
        }
        if let Some(boxed) = META_DEPS_STRINGS.take() {
            drop(boxed);
        }
        if let Some(boxed_ptrs) = META_DEPS_PTRS.take() {
            drop(boxed_ptrs);
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_on_load() {}

#[no_mangle]
pub extern "C" fn plugin_on_unload() {}

#[no_mangle]
pub extern "C" fn plugin_on_enable() {}

#[no_mangle]
pub extern "C" fn plugin_on_disable() {}

#[no_mangle]
pub extern "C" fn plugin_get_name() -> *const c_char {
    b"example_plugin\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn plugin_get_version() -> *const c_char {
    b"0.1.0\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn plugin_get_dependencies() -> *const *const c_char {
    unsafe {
        INIT_DEPS.call_once(|| {
            let mut deps: Vec<CString> = Vec::new();
            deps.push(CString::new("loader@^0.1.0").unwrap());

            let mut ptrs: Vec<*const c_char> = Vec::with_capacity(deps.len() + 1);
            for s in &deps {
                ptrs.push(s.as_ptr());
            }
            ptrs.push(std::ptr::null());

            META_DEPS_PTRS = Some(ptrs.into_boxed_slice());
            META_DEPS_STRINGS = Some(Box::new(deps));
        });
        META_DEPS_PTRS.as_ref().unwrap().as_ptr()
    }
}
