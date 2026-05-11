#[macro_export]
macro_rules! generate_abi {
    ($struct_name:ident, {
        $(
            $(#[$attr:meta])*
            $( optional )? fn $fn_name:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret:ty)?;
        )*
    }) => {
        /// Define a struct that holds all function pointers.
        pub struct $struct_name {
            // Define a dynamic library held in an Arc to allow sharing
            _lib: std::sync::Arc<libloading::Library>,
            $(
                $(#[$attr])*
                $fn_name: Option<unsafe extern "C" fn($($arg_ty),*) $(-> $ret)?>,
            )*
        }
        impl $struct_name {
            /// Load a dynamic library from a path and build the vtable.
            pub unsafe fn load(path: &str) -> Result<std::sync::Arc<Self>, Box<dyn std::error::Error>> {
                let lib = std::sync::Arc::new(unsafe { libloading::Library::new(path)? });
                unsafe { Self::load_from_lib(lib) }
            }

            /// Build the vtable from the loaded `Arc<Library>`.
            pub unsafe fn load_from_lib(lib: std::sync::Arc<libloading::Library>) -> Result<std::sync::Arc<Self>, Box<dyn std::error::Error>> {
                // Get symbols
                $(
                    let $fn_name = unsafe { (&*lib).get(stringify!($fn_name).as_bytes()) }.ok().map(|s| *s);
                )*

                let table = Self {
                    _lib: std::sync::Arc::clone(&lib),
                    $( $fn_name, )*
                };

                Ok(std::sync::Arc::new(table))
            }

            $(
                $crate::generate_abi_method! {
                    $(#[$attr])*
                    $fn_name($($arg: $arg_ty),*) $(-> $ret)?
                }
            )*
        }
    }
}

#[macro_export]
macro_rules! generate_abi_method {
    // return void
    ($(#[$attr:meta])* $fn_name:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty) => {
        $(#[$attr])*
        pub fn $fn_name(&self, $($arg: $arg_ty),*) -> Option<$ret> {
            if let Some(func) = self.$fn_name {
                unsafe {
                    Some(func($($arg),*))
                }
            } else {
                None
            }
        }
    };

    // return object
    ($(#[$attr:meta])* $fn_name:ident($($arg:ident: $arg_ty:ty),*)) => {
        $(#[$attr])*
        pub fn $fn_name(&self, $($arg: $arg_ty),*) {
            if let Some(func) = self.$fn_name {
                unsafe {
                    func($($arg),*)
                }
            }
        }
    };
}
