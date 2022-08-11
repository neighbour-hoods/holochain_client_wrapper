#![crate_type = "proc-macro"]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};

#[proc_macro_attribute]
pub fn generate_call(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let item_enum = syn::parse_macro_input!(item as syn::ItemEnum);
    let enum_name = item_enum.ident.to_string();
    println!("enum_name: {}", enum_name);
    (quote::quote! {
        #item_enum

        impl AdminWebsocket {
            pub async fn call(&self, cmd: AdminWsCmd) -> Result<JsValue, JsValue> {
                todo!()
            }
        }
    })
    .into()
}
