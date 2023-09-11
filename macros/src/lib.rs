mod query;
mod schema;
mod util;

use proc_macro::{self, TokenStream};
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    schema::schema_impl(input)
}

#[proc_macro_error]
#[proc_macro]
pub fn __query(input: TokenStream) -> TokenStream {
    query::query_impl(input)
}
