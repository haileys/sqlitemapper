use std::path::PathBuf;

use lazy_static::lazy_static;
use proc_macro_error::{abort_call_site, abort};
use proc_macro::{self, TokenStream};
use proc_macro2::{TokenStream as TokenStream2, Span, Ident};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::parse_macro_input;
use syn::parse::Parse;
use thiserror::Error;

use sqlitemapper_schema::{Schema, LoadError, SqlError, TableColumn};

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

struct SchemaInput {
    _opts: Punctuated<Opt, syn::token::Comma>,
    // item: ItemMod,
    // attrs: Attrs,
    // vis: syn::Visibility,
    // mod_token: syn::token::Mod,
    // ident: syn::Ident,
    // _semi: syn::token::Semi,
}

impl Parse for SchemaInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(SchemaInput {
            _opts: Punctuated::parse_terminated(input)?
        })
    }
}

enum Opt {
    // Path { value: LitStr },
}

impl Parse for Opt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        let _colon = input.parse::<syn::token::Colon>()?;
        match name.to_string().as_str() {
            // "path" => {
            //     Ok(Opt::Path {
            //         value: input.parse()?,
            //     })
            // }
            name => {
                abort!(name.span(), "unknown option in sqlitemapper::schema");
            }
        }
    }
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
    let _input = parse_macro_input!(input as SchemaInput);

    let schema = current();

    let schema_content = match schema_mod_content(&schema) {
        Ok(content) => content,
        Err(e) => { abort_call_site!("{}", e); }
    };

    schema_content.into()
}

fn schema_mod_content(schema: &Schema)
    -> Result<TokenStream2, SchemaMacroError>
{
    let tables = schema.tables()
        .map_err(SchemaMacroError::ListTables)?;

    let table_mods = tables.iter()
        .map(|table| {
            table_mod(schema, table)
                .map_err(|error| {
                    let table = table.clone();
                    SchemaMacroError::LoadTableSchema { table, error }
                })
        })
        .collect::<Result<TokenStream2, SchemaMacroError>>()?;

    Ok(table_mods)
}

fn table_mod(schema: &Schema, table: &str)
    -> Result<TokenStream2, SqlError>
{
    let columns = schema.columns(table)?;

    let type_aliases = columns.iter()
        .map(table_column_type_alias)
        .collect::<TokenStream2>();

    let primary_key = table_primary_key(&columns);

    let table_name = Ident::new_raw(table, Span::mixed_site());

    Ok(quote! {
        pub mod #table_name {
            #type_aliases
            #primary_key
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

fn table_primary_key(columns: &[TableColumn]) -> TokenStream2 {
    let mut pkeys = columns.iter()
        .filter(|column| column.primary_key_part.is_some())
        .collect::<Vec<_>>();

    pkeys.sort_by_key(|col| col.primary_key_part);

    // don't generate a pkey type if there are no pkeys
    if pkeys.len() == 0 {
        return quote!{}
    }

    let fields = pkeys.iter()
        .map(|pkey| {
            let name = Ident::new_raw(&pkey.name, Span::mixed_site());
            quote! { pub #name, }
        })
        .collect::<TokenStream2>();

    quote! {
        // #[derive(Debug, Clone, PartialEq, PartialOrd)]
        pub struct Id(#fields);
    }
}
