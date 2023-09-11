use derive_syn_parse::Parse;
use proc_macro::{self, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{parse_macro_input, LitStr};

use sqlitemapper_schema::{QueryInfo, ResultColumn};

#[derive(Parse)]
struct QueryInput {
    query: LitStr,
}

pub fn query_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as QueryInput);
    let query = input.query;

    let info = prepare_query(&query.value());

    let columns = quote_columns_vec(info.columns());

    let output: TokenStream2 = quote! {
        ::sqlitemapper::Query {
            sql: #query,
            types: vec![#columns],
        }
    };

    output.into()
}

fn quote_columns_vec(columns: &[ResultColumn]) -> TokenStream2 {
    let elements = columns.iter()
        .map(quote_column_ctor)
        .map(|expr| quote!{ #expr , })
        .collect::<TokenStream2>();

    quote! { ::std::vec![ #elements ] }
}

fn quote_column_ctor(_: &ResultColumn) -> TokenStream2 {
    // let sql_type = match column.sql_type() {
    //     None => quote! { ::core::option::Option::None },
    //     Some(ty) => {
    //         let lit = LitStr::new(ty, Span::mixed_site());
    //         quote! { ::core::option::Option::Some(#lit) }
    //     }
    // };

    quote! {
        ::sqlitemapper::Column {
            // sql_type: #sql_type,
        }
    }
}

fn prepare_query(query: &str) -> QueryInfo {
    match crate::schema::current().prepare(query) {
        Ok(info) => info,
        Err(e) => abort_call_site!("{}", e),
    }
}
