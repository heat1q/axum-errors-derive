use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Error)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl ::std::error::Error for #ident {}

        impl ::std::convert::From<::axum::extract:::rejection::JsonRejection> for #ident {}
    };

    output.into()
}
