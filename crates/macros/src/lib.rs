#![crate_type = "proc-macro"]

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};

#[proc_macro]
pub fn generate_call(input: TokenStream) -> TokenStream {
    (quote::quote! {
        pub async fn call(&self, cmd: AdminWsCmd) -> Result<JsValue, JsValue> {
            todo!()
        }
    })
    .into()
}
