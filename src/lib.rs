use proc_macro::{self, TokenStream};
use syn::Error;

mod from_rejection;

#[proc_macro_derive(FromRejection, attributes(status_code, rejection))]
pub fn derive_from_rejection(input: TokenStream) -> TokenStream {
    syn::parse(input)
        .and_then(from_rejection::expand_from_rejection)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
