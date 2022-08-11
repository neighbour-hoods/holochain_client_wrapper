#![crate_type = "proc-macro"]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use syn::{Field, Fields, spanned::Spanned, punctuated::Punctuated, token::Comma};

#[proc_macro_attribute]
pub fn generate_call(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item_enum = syn::parse_macro_input!(item as syn::ItemEnum);
    let enum_name = item_enum.ident.clone();

    let mut match_blocks = TokenStream2::new();
    for variant in &item_enum.variants {
        let variant_name = variant.ident.clone();
        let variant_name_camel_case = lowercase_first_letter(variant.ident.to_string());

        let variant_fields_ident_vec: Vec<Ident> = match &variant.fields {
            Fields::Unnamed(_) => panic!("unnamed fields are not allowed"),
            // TODO we need to handle the unit case differently - not as a map, just a unit type
            Fields::Unit => vec![],
            Fields::Named(fields_named) => fields_named.named.iter().map(|field| field.ident.clone().expect("field should have ident")).collect(),
        };
        let variant_fields_ident_comma_punctuated: Punctuated<Ident, Comma> = {
            let mut ret = Punctuated::new();
            for ident in variant_fields_ident_vec.iter().cloned() {
                ret.push(ident);
            }
            ret
        };

        let field_insertion_blob: TokenStream2 = {
            let mut field_insertion_blob = TokenStream2::new();
            for field_ident in variant_fields_ident_vec {
                field_insertion_blob.extend(quote::quote_spanned! {variant.span() =>
                    assert!(Reflect::set(
                        &payload,
                        &(stringify!(#field_ident).into()),
                        // TODO here we might consider adding a Trait to handle custom
                        // converting-to-object for types we can then offer as strongly-typed
                        // within the `AdminWsCmd` enum.
                        //
                        // `.into()` will only work for simple Rust types...
                        &(#field_ident.into()),
                    )?);
                });
            }
            field_insertion_blob
        };

        match_blocks.extend(quote::quote_spanned! {variant.span()=>
            #enum_name::#variant_name { #variant_fields_ident_comma_punctuated } => {
                let tag: &str = #variant_name_camel_case;
                let method: Function = Reflect::get(&self.js_ws, &JsValue::from_str(&tag))?.dyn_into()?;
                let payload: JsValue = {
                    let payload: JsValue = Object::new().dyn_into()?;
                    assert!(Reflect::set(
                        &payload,
                        &JsValue::from_str("tag"),
                        &JsValue::from_str(tag)
                    )?);
                    #field_insertion_blob
                    payload
                };
                let promise: Promise = method.call1(&self.js_ws, &payload)?.dyn_into()?;
                let future: JsFuture = promise.into();
                future.await
            }
        });
    }

    (quote::quote! {
        #item_enum

        impl AdminWebsocket {
            pub async fn call(&self, cmd: AdminWsCmd) -> Result<JsValue, JsValue> {
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
