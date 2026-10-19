use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::token::Comma;
use syn::{
    Ident, Result, Type,
    parse::{Parse, ParseStream},
};

struct FfiObjectMacroInput {
    wrapper_name: Ident,
    target_ty: Type,
    free_fn_name: Ident,
}

impl Parse for FfiObjectMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let wrapper_name: Ident = input.parse()?;
        input.parse::<Comma>()?;
        let target_ty: Type = input.parse()?;
        input.parse::<Comma>()?;
        let free_fn_name: Ident = input.parse()?;

        Ok(FfiObjectMacroInput {
            wrapper_name,
            target_ty,
            free_fn_name,
        })
    }
}

pub fn generate_macro(input: TokenStream2) -> TokenStream2 {
    let parsed: FfiObjectMacroInput = match syn::parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error(),
    };

    let wrapper_name = parsed.wrapper_name;
    let target_ty = parsed.target_ty;
    let free_fn_name = parsed.free_fn_name;
    let free_fn_name_str = format!("{}\0", free_fn_name);

    quote! {
        pub struct #wrapper_name {
            ptr: Option<#target_ty>,
            free_fn: unsafe extern "C" fn(#target_ty),
            lib: std::sync::Arc<libloading::Library>,
        }

        impl #wrapper_name {
            pub fn new(ptr: #target_ty, lib: std::sync::Arc<libloading::Library>) -> Self {
                let free_fn: unsafe extern "C" fn(#target_ty) = unsafe {
                    let sym: libloading::Symbol<unsafe extern "C" fn(#target_ty)> = lib
                        .get(#free_fn_name_str.as_bytes())
                        .unwrap();
                    *sym
                };
                Self {
                    ptr: Some(ptr),
                    free_fn,
                    lib,
                }
            }

            pub fn get(&self) -> &#target_ty {
                self.ptr.as_ref().expect("The raw object is already taken")
            }

            pub fn into_inner(mut self) -> #target_ty {
                let ptr = self.ptr.take().expect("The raw object is already taken");
                std::mem::forget(self);
                ptr
            }
        }

        impl Drop for #wrapper_name {
            fn drop(&mut self) {
                if let Some(ptr) = self.ptr.take() {
                    unsafe {
                        (self.free_fn)(ptr);
                    }
                }
            }
        }

        impl plugin_api::FfiWrapper for #wrapper_name {
            type Raw = #target_ty;
            fn from_raw(raw: Self::Raw, lib: std::sync::Arc<libloading::Library>) -> Self {
                Self::new(raw, lib)
            }
        }
    }
}
