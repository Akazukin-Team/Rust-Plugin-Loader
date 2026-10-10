use proc_macro2::TokenStream;
use quote::quote;
use syn::token::{Comma, Fn, RArrow, Semi};
use syn::{
    Attribute, Ident, Result, Type,
    parse::{Parse, ParseStream},
};

struct VTableReturnType {
    raw_ty: Type,
    wrapper_ty: Type,
}

struct VTableMethod {
    attrs: Vec<Attribute>,
    is_optional: bool,
    fn_ident: Ident,
    args: Vec<FnArg>,
    ret: Option<VTableReturnType>,
}

impl Parse for VTableMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = Attribute::parse_outer(input)?;

        let is_optional = if input.peek(Ident) && input.fork().parse::<Ident>()? == "optional" {
            let _: Ident = input.parse()?;
            true
        } else {
            false
        };

        input.parse::<Fn>()?;
        let fn_ident: Ident = input.parse()?;

        let content;
        syn::parenthesized!(content in input);
        let mut args = Vec::new();
        while !content.is_empty() {
            let name: Ident = content.parse()?;
            content.parse::<syn::Token![:]>()?;
            let ty: Type = content.parse()?;
            args.push(FnArg { name, ty });
            if content.peek(Comma) {
                content.parse::<Comma>()?;
            }
        }

        let mut ret: Option<VTableReturnType> = None;

        if input.peek(RArrow) {
            input.parse::<RArrow>()?;
            let ret_content;
            syn::parenthesized!(ret_content in input);
            let raw_ty = ret_content.parse()?;
            ret_content.parse::<RArrow>()?;
            let wrapper_ty = ret_content.parse()?;
            ret = Some(VTableReturnType {
                raw_ty,
                wrapper_ty,
            });
        }

        input.parse::<Semi>()?;

        Ok(VTableMethod {
            attrs,
            is_optional,
            fn_ident,
            args,
            ret,
        })
    }
}

struct FnArg {
    name: Ident,
    ty: Type,
}

struct VTableMacroInput {
    struct_ident: Ident,
    methods: Vec<VTableMethod>,
}

impl Parse for VTableMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_ident: Ident = input.parse()?;
        input.parse::<Comma>()?;

        let content;
        syn::braced!(content in input);
        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(content.parse()?);
        }

        Ok(VTableMacroInput {
            struct_ident,
            methods,
        })
    }
}

pub fn generate_macro(input: TokenStream) -> TokenStream {
    let input: VTableMacroInput = match syn::parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error(),
    };

    let struct_ident = input.struct_ident;

    let mut vtable_fields = Vec::new();
    let mut field_inits = Vec::new();
    let mut load_checks = Vec::new();
    let mut method_impls = Vec::new();

    for method in input.methods {
        let fn_ident = method.fn_ident;
        let fn_name_str = fn_ident.to_string();
        let attrs = method.attrs;
        let is_optional = method.is_optional;

        let arg_names: Vec<Ident> = method.args.iter().map(|a| a.name.clone()).collect();
        let arg_types: Vec<Type> = method.args.iter().map(|a| a.ty.clone()).collect();

        let call_func = if method.is_optional {
            quote! {
                unsafe { func(#(#arg_names),*) }
            }
        } else {
            quote! {
                unsafe { (self.#fn_ident)(#(#arg_names),*) }
            }
        };

        let (raw_ty, wrap_ty, inner_body) = if let Some(ret) = &method.ret {
            let raw_ty = &ret.raw_ty;
            let wrap_ty = &ret.wrapper_ty;

            (
                quote! { #raw_ty },
                quote! { #wrap_ty },
                quote! {
                        let raw_ret = #call_func;
                        plugin_api::FfiWrapper::from_raw(raw_ret, self.library())
                },
            )
        } else {
            (quote! { () }, quote! { () }, call_func)
        };

        let (field_ty, call) = if is_optional {
            (
                quote! { Option<unsafe extern "C" fn(#(#arg_types),*) -> #raw_ty> },
                quote! {
                    #(#attrs)*
                    pub fn #fn_ident(&self) -> Option<impl Fn(#(#arg_types),*) -> #wrap_ty + '_> {
                        self.#fn_ident.as_ref().map(|func| {
                            move || {
                                #inner_body
                            }
                        })
                    }
                },
            )
        } else {
            (
                quote! { unsafe extern "C" fn(#(#arg_types),*) -> #raw_ty },
                quote! {
                    #(#attrs)*
                    pub fn #fn_ident(&self) -> impl Fn(#(#arg_types),*) -> #wrap_ty + '_ {
                       move || {
                            #inner_body
                        }
                    }
                },
            )
        };

        let field_init = if is_optional {
            quote! { raw_fn }
        } else {
            quote! {
                raw_fn.ok_or(VTableError::MissingFunction(#fn_name_str))?
            }
        };

        vtable_fields.push(quote! {
            pub #fn_ident:  #field_ty
        });

        let c_str_name = format!("{}\0", fn_name_str);
        load_checks.push(quote! {
            let raw_fn = unsafe { (&*lib).get(#c_str_name.as_bytes()) }.ok().map(|s| *s);
            let #fn_ident = #field_init;
        });

        field_inits.push(quote! {
            #fn_ident
        });

        method_impls.push(call);
    }

    let expanded = quote! {
        pub struct #struct_ident {
            _lib: std::sync::Arc<libloading::Library>,
            #(#vtable_fields,)*
        }

        impl #struct_ident {
            pub fn load(path: &str) -> Result<std::sync::Arc<Self>, libloading::Error> {
                let lib = std::sync::Arc::new(unsafe { libloading::Library::new(path)? });
                Self::load_from_lib(lib)
            }

            pub fn load_from_lib(lib: std::sync::Arc<libloading::Library>) -> Result<std::sync::Arc<Self>, libloading::Error> {
                #(#load_checks)*

                let table = Self {
                    _lib: std::sync::Arc::clone(&lib),
                    #(#field_inits,)*
                };

                Ok(std::sync::Arc::new(table))
            }

            pub fn library(&self) -> std::sync::Arc<libloading::Library> {
                std::sync::Arc::clone(&self._lib)
            }

            #(#method_impls)*
        }
    };

    expanded.into()
}
