use derive_syn_parse::Parse;
use proc_macro::{self, TokenStream};
use proc_macro2::{TokenStream as TokenStream2, Ident, Span};
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{parse_macro_input, LitStr};

use sqlitemapper_schema::{QueryInfo, ResultColumn};

#[derive(Parse)]
struct QueryInput {
    schema: syn::Path,
    _comma: syn::token::Comma,
    query: LitStr,
}

pub fn query_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryInput);
    let query = input.query;

    let info = prepare_query(&query.value());

    let row_type = row_type(&input.schema, &info);

    let output: TokenStream2 = quote! {
        ::sqlitemapper::Query::<#row_type>::new_unchecked(#query)
    };

    output.into()
}

fn row_type(schema: &syn::Path, info: &QueryInfo) -> TokenStream2 {
    info.columns()
        .iter()
        .rev()
        .map(|col| column_type(schema, col))
        .fold(quote! { () }, |tail, ty| {
            quote!{ (#ty, #tail) }
        })
}

fn column_type(schema: &syn::Path, column: &ResultColumn) -> TokenStream2 {
    let (Some(table_name), Some(column_name), Some(schema_name))
        = (column.origin_table(), column.origin_column(), column.origin_database())
        else {
            let name = column.describe();
            abort_call_site!("{} is an expression, this is unsupported", name);
        };

    if schema_name != "main" {
        let name = column.describe();
        abort_call_site!("{} is from foreign schema {}, this is unsupported", name, schema_name);
    }

    let table = Ident::new_raw(table_name, Span::mixed_site());
    let column = Ident::new_raw(column_name, Span::mixed_site());

    quote! { #schema::#table::#column }
}

fn prepare_query(query: &str) -> QueryInfo {
    match crate::schema::current().prepare(query) {
        Ok(info) => info,
        Err(e) => abort_call_site!("{}", e),
    }
}
