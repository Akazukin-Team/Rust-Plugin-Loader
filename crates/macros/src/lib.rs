use proc_macro::TokenStream;

mod generate_ffi_object;
mod generate_vtable;

#[proc_macro]
pub fn generate_vtable(input: TokenStream) -> TokenStream {
    let input2 = proc_macro2::TokenStream::from(input);
    let output = generate_vtable::generate_macro(input2);
    TokenStream::from(output)
}

#[proc_macro]
pub fn generate_ffi_object(input: TokenStream) -> TokenStream {
    let input2 = proc_macro2::TokenStream::from(input);
    let output = generate_ffi_object::generate_macro(input2);
    TokenStream::from(output)
}
