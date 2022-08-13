#![crate_type = "proc-macro"]

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use syn::{parse::Parser, punctuated::Punctuated, spanned::Spanned, token::Comma, Fields};

#[proc_macro_attribute]
pub fn generate_call(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let mut attrs_ident_iter = match Punctuated::<Ident, Comma>::parse_terminated.parse(attrs) {
        Err(err) => panic!("attrs_parsed: err: {}", err),
        Ok(x) => x.into_iter(),
    };
    let ident_ws = attrs_ident_iter.next().expect("ident_ws to be passed");
    let ident_ws_cmd = attrs_ident_iter.next().expect("ident_ws_cmd to be passed");
    let ident_ws_cmd_resp = attrs_ident_iter
        .next()
        .expect("ident_ws_cmd_resp to be passed");
    let ident_parse_resp = attrs_ident_iter
        .next()
        .expect("ident_parse_resp to be passed");

    let item_enum = syn::parse_macro_input!(item as syn::ItemEnum);
    let enum_name = item_enum.ident.clone();

    let mut match_blocks = TokenStream2::new();
    for variant in &item_enum.variants {
        let variant_name = variant.ident.clone();
        let variant_name_camel_case = lowercase_first_letter(variant.ident.to_string());

        let (method_call_tokenstream, enum_match_binder): (TokenStream2, TokenStream2) =
            match &variant.fields {
                Fields::Unnamed(_) => panic!("unnamed fields are not allowed"),
                Fields::Unit => {
                    let method_call_tokenstream: TokenStream2 = quote::quote! {
                        let promise: Promise = method.call0(&self.js_ws)?.dyn_into()?;
                    };

                    let enum_match_binder: TokenStream2 = quote::quote! {
                        #variant_name
                    };

                    (method_call_tokenstream, enum_match_binder)
                }
                Fields::Named(fields_named) => {
                    let variant_fields_ident_vec: Vec<Ident> = fields_named
                        .named
                        .iter()
                        .map(|field| field.ident.clone().expect("field should have ident"))
                        .collect();
                    let variant_fields_ident_comma_punctuated: Punctuated<Ident, Comma> =
                        Punctuated::from_iter(variant_fields_ident_vec.iter().cloned());

                    let field_insertion_blob: TokenStream2 = {
                        let mut field_insertion_blob = TokenStream2::new();
                        for field_ident in variant_fields_ident_vec {
                            field_insertion_blob.extend(quote::quote_spanned! {variant.span() =>
                                assert!(Reflect::set(
                                    &payload,
                                    &(stringify!(#field_ident).into()),
                                    // TODO here we might consider adding a Trait to handle custom
                                    // converting-to-object for types we can then offer as strongly-typed
                                    // within the `AdminWsCmd`/`AppWsCmd` enum.
                                    //
                                    // `.into()` will only work for simple Rust types...
                                    &(#field_ident.into()),
                                )?);
                            });
                        }
                        field_insertion_blob
                    };

                    let method_call_tokenstream: TokenStream2 = quote::quote! {
                        let payload: JsValue = {
                            let payload: JsValue = Object::new().dyn_into()?;
                            #field_insertion_blob
                            payload
                        };
                        let promise: Promise = method.call1(&self.js_ws, &payload)?.dyn_into()?;
                    };

                    let enum_match_binder: TokenStream2 = quote::quote! {
                        #variant_name { #variant_fields_ident_comma_punctuated }
                    };

                    (method_call_tokenstream, enum_match_binder)
                }
            };

        match_blocks.extend(quote::quote_spanned! {variant.span()=>
            #enum_name::#enum_match_binder => {
                let method: Function = Reflect::get(&self.js_ws, &JsValue::from_str(#variant_name_camel_case))?.dyn_into()?;
                #method_call_tokenstream
                let future: JsFuture = promise.into();
                future.await
                    .map(|val| #ident_parse_resp(val, stringify!(#variant_name).into()))
            }
        });
    }

    (quote::quote! {
        #item_enum

        impl #ident_ws {
            pub async fn call(&self, cmd: #ident_ws_cmd) -> Result<#ident_ws_cmd_resp, JsValue> {
                match cmd {
                    #match_blocks
                }
            }
        }
    })
    .into()
}

fn lowercase_first_letter(s: String) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}
