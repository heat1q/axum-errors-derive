use proc_macro2::{self, TokenStream};
use quote::quote;
use syn::ItemStruct;

pub(crate) fn expand_from_rejection(item: ItemStruct) -> syn::Result<TokenStream> {
    expand_struct(item)
}

fn expand_struct(item: ItemStruct) -> syn::Result<TokenStream> {
    let ident = &item.ident;
    let Some(status_code) = field_for_attr(&item, "status_code") else {
        panic!("`status_code` not provided")
    };
    let status_code_ident = &status_code.ident;

    let Some(reason) = field_for_attr(&item, "reason") else {
        panic!("`reason` not provided")
    };
    let reason_ident = &reason.ident;

    let from_byte_rej = quote! {
        impl ::std::convert::From<::axum::extract::rejection::BytesRejection> for #ident {
            fn from(byte_rej: ::axum::extract::rejection::BytesRejection) -> Self {
                let (status_code, reason) = match &byte_rej {
                    ::axum::extract::rejection::BytesRejection::FailedToBufferBody(buf_rej) => match &buf_rej {
                        ::axum::extract::rejection::FailedToBufferBody::LengthLimitError(err) => (::axum::http::StatusCode::PAYLOAD_TOO_LARGE, err.to_string()),
                        ::axum::extract::rejection::FailedToBufferBody::UnknownBodyError(err) => (::axum::http::StatusCode::BAD_REQUEST, err.to_string()),
                    },
                };

                Self {
                    #status_code_ident: status_code,
                    #reason_ident: ::std::option::Option::Some(reason),
                    ..Self::default()
                }
            }
        }
    };

    let output = quote! {
        #from_byte_rej
    };

    Ok(output)
}

fn field_for_attr<'a>(item: &'a ItemStruct, attr_ident: &'a str) -> Option<&'a syn::Field> {
    for field in &item.fields {
        for attr in &field.attrs {
            if !attr.path.is_ident("rejection") {
                continue;
            }

            let args: syn::Path = attr.parse_args().unwrap();
            if args.is_ident(attr_ident) {
                return Some(field);
            }
        }
    }

    None
}
