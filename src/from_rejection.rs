use proc_macro2::{self, TokenStream};
use quote::quote;
use syn::ItemStruct;

pub(crate) fn expand_from_rejection(item: ItemStruct) -> syn::Result<TokenStream> {
    expand_struct(item)
}

fn expand_struct(item: ItemStruct) -> syn::Result<TokenStream> {
    let Some(status_code) = field_for_attr(&item, "status_code") else {
        panic!("`status_code` not provided")
    };
    let Some(reason) = field_for_attr(&item, "reason") else {
        panic!("`reason` not provided")
    };

    let from_byte_rej = impl_from_byte_rej(&item, status_code, reason);
    let output = quote! {
        #from_byte_rej
    };

    Ok(output)
}

fn impl_from_byte_rej(
    item: &ItemStruct,
    status_code: &syn::Field,
    reason: &syn::Field,
) -> TokenStream {
    let byte_rej = quote!(::axum::extract::rejection::BytesRejection);
    let buf_body_rej = quote!(::axum::extract::rejection::FailedToBufferBody);
    let some = quote!(::std::option::Option::Some);

    let from_fn_match = quote! {
        match &byte_rej {
            #byte_rej::FailedToBufferBody(buf_rej) => match &buf_rej {
                #buf_body_rej::LengthLimitError(err) => (err.status(), #some(err.body_text())),
                #buf_body_rej::UnknownBodyError(err) => (err.status(), #some(err.body_text())),
                &_ => (::axum::http::StatusCode::BAD_REQUEST, ::std::option::Option::None),
            },
            &_ => (::axum::http::StatusCode::BAD_REQUEST, ::std::option::Option::None),
        }
    };

    impl_from(&item.ident, status_code, reason, &byte_rej, &from_fn_match)
}

fn impl_from(
    ty: &syn::Ident,
    status_code_field: &syn::Field,
    reason_field: &syn::Field,
    rej_ty: &TokenStream,
    from_fn_match: &TokenStream,
) -> TokenStream {
    let status_code_ident = &status_code_field.ident;
    let reason_ident = &reason_field.ident;

    quote! {
        impl ::std::convert::From<#rej_ty> for #ty {
            fn from(byte_rej: #rej_ty) -> Self {
                let (status_code, reason) = #from_fn_match;
                Self {
                    #status_code_ident: status_code.into(),
                    #reason_ident: reason.map(|x| x.into()),
                    ..Self::default()
                }
            }
        }
    }
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
