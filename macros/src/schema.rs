use std::path::PathBuf;

use derive_syn_parse::Parse;
use lazy_static::lazy_static;
use proc_macro_error::abort_call_site;
use proc_macro::{self, TokenStream};
use proc_macro2::{TokenStream as TokenStream2, Span, Ident};
use quote::quote;
use syn::parse_macro_input;
use thiserror::Error;

use sqlitemapper_schema::{Schema, LoadError, SqlError, TableColumn};

use crate::util::Attrs;

lazy_static! {
    static ref SCHEMA: Result<Schema, SchemaError> = load_from_env();
}

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("SQLITEMAPPER_SCHEMA environment variable not set")]
    EnvVarNotSet,
    #[error("Error loading schema from {path}: {error}")]
    Load { path: PathBuf, error: LoadError },
}

pub fn try_current() -> Result<&'static Schema, &'static SchemaError> {
    SCHEMA.as_ref()
}

pub fn current() -> &'static Schema {
    match try_current() {
        Ok(schema) => schema,
        Err(e) => abort_call_site!("{}", e),
    }
}

fn load_from_env() -> Result<Schema, SchemaError> {
    let path = std::env::var_os("SQLITEMAPPER_SCHEMA")
        .ok_or(SchemaError::EnvVarNotSet)?;

    let path = PathBuf::from(path);

    Schema::from_file(&path)
        .map_err(|error| SchemaError::Load { path, error })
}

#[derive(Parse)]
struct SchemaInput {
    attrs: Attrs,
    vis: syn::Visibility,
    mod_token: syn::token::Mod,
    ident: syn::Ident,
    _semi: syn::token::Semi,
}

#[derive(Error, Debug)]
pub enum SchemaMacroError {
    #[error("Error listing tables: {0}")]
    ListTables(SqlError),
    #[error("Error loading schema for table {table}: {error}")]
    LoadTableSchema { table: String, error: SqlError },
}

pub fn schema_impl(input: TokenStream)
    -> TokenStream
{
    let input = parse_macro_input!(input as SchemaInput);

    let attrs = input.attrs;
    let vis = input.vis;
    let mod_token = input.mod_token;
    let ident = input.ident;

    let schema = current();
    let content = match schema_mod_content(schema) {
        Ok(content) => content,
        Err(e) => { abort_call_site!("{}", e); }
    };

    let mod_defn = quote! {
        #attrs #vis #mod_token #ident { #content }
    };

    mod_defn.into()
}

fn schema_mod_content(schema: &Schema)
    -> Result<TokenStream2, SchemaMacroError>
{
    let tables = schema.tables()
        .map_err(SchemaMacroError::ListTables)?;

    let mut stream = TokenStream2::default();

    for table in tables {
        let tokens = table_mod(schema, &table)
            .map_err(|error| {
                SchemaMacroError::LoadTableSchema { table, error }
            })?;

        stream.extend(tokens);
    }

    Ok(stream)
}

fn table_mod(schema: &Schema, table: &str)
    -> Result<TokenStream2, SqlError>
{
    let columns = schema.columns(table)?;

    let type_aliases = columns.iter()
        .map(table_column_type_alias)
        .collect::<TokenStream2>();

    let table_name = Ident::new_raw(table, Span::mixed_site());

    Ok(quote! {
        pub mod #table_name {
            #type_aliases
        }
    })
}

fn table_column_type_alias(column: &TableColumn) -> TokenStream2 {
    let inherent_type = match column.type_.as_str() {
        "INT" | "INTEGER" => quote! { ::core::primitive::i64 },
        "REAL" => quote! { ::core::primitive::f64 },
        "TEXT" => quote! { ::std::string::String },
        "BLOB" => quote! { ::std::vec::Vec<::core::u8> },
        _ => { abort_call_site!("unknown sqlite datatype: {}", column.type_); }
    };

    let type_ = match column.not_null {
        true => inherent_type,
        false => quote! { ::core::option::Option<#inherent_type> },
    };

    let name = Ident::new_raw(&column.name, Span::mixed_site());

    quote! { pub type #name = #type_; }
}
