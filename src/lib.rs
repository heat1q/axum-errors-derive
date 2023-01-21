use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Ident};

#[proc_macro_derive(FromRejection, attributes(status_code, rejection))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let ident = &input.ident;
    match input.data {
        Data::Struct(input) => impl_struct(ident, input),
        _ => panic!("derive FromRejection is only supported on structs", input),
    }
}

fn impl_struct(ident: &Ident, data: DataStruct) -> TokenStream {
    let Some(status_code) = field_for_attr(&data, "status_code") else {
        panic!("`status_code` not provided")
    };
    let status_code_ident = &status_code.ident;

    let Some(reason) = field_for_attr(&data, "reason") else {
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

    output.into()
}

fn field_for_attr<'a>(data: &'a DataStruct, attr_ident: &'a str) -> Option<&'a syn::Field> {
    if let syn::Fields::Named(fields) = &data.fields {
        for named_field in &fields.named {
            for attr in &named_field.attrs {
                let meta = attr.parse_meta().unwrap();

                let Some(meta_ident) = meta.path().get_ident() else {
                    continue;
                };

                if meta_ident != "rejection" {
                    continue;
                }

                let meta_list = match &meta {
                    syn::Meta::List(meta_list) => meta_list,
                    _ => continue,
                };

                for nested_meta in &meta_list.nested {
                    let m = match nested_meta {
                        syn::NestedMeta::Meta(m) => m,
                        _ => continue,
                    };

                    println!("{:?}", m.path().get_ident());

                    if let Some(ident) = m.path().get_ident() {
                        println!("{:?} == {}", m.path().get_ident(), attr_ident);
                        if ident == attr_ident {
                            return Some(named_field);
                        }
                    }
                }
            }
        }
    }

    None
}
