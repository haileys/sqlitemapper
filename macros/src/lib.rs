mod query;
mod schema;
mod util;

use proc_macro::{self, TokenStream};
use proc_macro_error::proc_macro_error;
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro_error]
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as schema::SchemaInput);
    schema::schema_impl(input).into_token_stream().into()
}

#[proc_macro_error]
#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    query::query_impl(input)
}
