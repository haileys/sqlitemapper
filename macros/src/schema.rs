use std::collections::HashMap;
use std::path::PathBuf;

use lazy_static::lazy_static;
use proc_macro_error::{abort_call_site, emit_error, emit_warning};
use proc_macro2::{TokenStream as TokenStream2, Span, Ident, Group, Delimiter};
use quote::spanned::Spanned;
use quote::{quote, ToTokens};
use syn::token::Brace;
use syn::{ItemMod, ItemType, Type, Visibility};
use syn::{Item, parse_quote};
use thiserror::Error;

use sqlitemapper_schema::{Schema, LoadError, TableColumn};

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

pub type SchemaInput = ItemMod;

pub fn schema_impl(input: ItemMod) -> ItemMod {
    let schema = current();
    let schema_mod_decl = parse_schema_mod(input);
    generate_schema_mod(schema, schema_mod_decl)
}

struct SchemaModDecl {
    item: ItemMod,
    table_mods: HashMap<String, TableModDecl>,
    unknown_items: Vec<Item>,
}

fn parse_schema_mod(mut item: ItemMod) -> SchemaModDecl {
    let mut table_mods = HashMap::new();
    let mut unknown_items = Vec::new();

    let items = item.content.take()
        .into_iter()
        .flat_map(|(_, items)| items);

    for item in items {
        match item {
            Item::Mod(item) => {
                let name = item.ident.to_string();

                if table_mods.contains_key(&name) {
                    emit_error!(item.__span(), "Duplicate mod definition");
                    continue;
                }

                table_mods.insert(name, parse_table_mod(item));
            }
            _ => {
                emit_error!(item.__span(), "Only table mods allowed in schema mod");
                unknown_items.push(item);
            }
        }
    }

    SchemaModDecl { item, table_mods, unknown_items }
}

#[derive(Default)]
struct TableModDecl {
    ident: Option<Ident>,
    column_type_aliases: HashMap<String, ColumnTypeAliasDecl>,
    unknown_items: Vec<Item>,
}

fn parse_table_mod(mut item: ItemMod) -> TableModDecl {
    for attr in item.attrs {
        emit_error!(attr.__span(), "Attributes not allowed on table mods");
    }

    match item.vis {
        Visibility::Inherited => {}
        Visibility::Public(pub_) => {
            emit_warning!(pub_.span, "Unnecessary pub keyboard, table mods are always public");
        }
        Visibility::Restricted(restrict) => {
            emit_error!(restrict.__span(), "Restricted visibility not allowed on table mods, set visibility on top level schema mod instead");
        }
    }

    let items = item.content
        .take()
        .map(|(_, items)| items)
        .unwrap_or_default();

    let mut column_type_aliases = HashMap::default();
    let mut unknown_items = Vec::default();

    for item in items {
        match item {
            Item::Type(item) => {
                let name = item.ident.to_string();

                if column_type_aliases.contains_key(&name) {
                    emit_error!(item.__span(), "Duplicate type definition");
                }

                column_type_aliases.insert(name,
                    parse_column_type_alias(item));
            }
            _ => {
                emit_error!(item.__span(), "Only column types allowed in table mod");
                unknown_items.push(item);
            }
        }
    }

    TableModDecl {
        ident: Some(item.ident),
        column_type_aliases,
        unknown_items,
    }
}

struct ColumnTypeAliasDecl {
    ty: Box<Type>,
}

fn parse_column_type_alias(item: ItemType) -> ColumnTypeAliasDecl {
    for attr in &item.attrs {
        emit_error!(attr.__span(), "Attributes not allowed on column types");
    }

    match &item.vis {
        Visibility::Inherited => {}
        Visibility::Public(pub_) => {
            emit_warning!(pub_.span, "Unnecessary pub keyboard, column types are always public");
        }
        Visibility::Restricted(restrict) => {
            emit_error!(restrict.__span(), "Restricted visibility not allowed on column types, set visibility on top level schema mod instead");
        }
    }

    if item.generics.lt_token.is_some() || item.generics.where_clause.is_some() {
        emit_error!(item.__span(), "Generics not allowed on column types");
    }

    ColumnTypeAliasDecl {
        ty: item.ty,
    }
}

fn generate_schema_mod(schema: &Schema, mut decl: SchemaModDecl) -> ItemMod {
    let tables = schema.tables().unwrap_or_else(|err| {
        abort_call_site!("Error listing SQLite tables: {}", err);
    });

    let (brace, mut items) = decl.item.content
        .map(|(brace, items)| (Some(brace), items))
        .unwrap_or_default();

    for table in tables {
        let table_decl = decl.table_mods.remove(&table);
        let table_mod = generate_table_mod(schema, &table, table_decl);
        items.push(Item::Mod(table_mod));
    }

    for (name, table_decl) in decl.table_mods {
        let span = table_decl.ident.__span();
        emit_error!(span, "No table {:?} found, only mods corresponding to SQLite tables allowed in schema mod", name);
    }

    items.extend(decl.unknown_items);

    let tokens = items.iter()
        .map(|item| item.into_token_stream())
        .collect();

    let group = Group::new(Delimiter::Brace, tokens);

    let brace = brace.unwrap_or(Brace { span: group.delim_span() });

    ItemMod {
        attrs: decl.item.attrs,
        vis: decl.item.vis,
        unsafety: decl.item.unsafety,
        mod_token: decl.item.mod_token,
        ident: decl.item.ident,
        content: Some((brace, items)),
        semi: None,
    }
}

fn token_stream<T: ToTokens>(items: impl IntoIterator<Item = T>) -> TokenStream2 {
    items.into_iter()
        .map(|item| item.to_token_stream())
        .collect()
}

fn generate_table_mod(schema: &Schema, table: &str, mut decl: Option<TableModDecl>) -> ItemMod {
    let columns = schema.columns(table).unwrap_or_else(|err| {
        abort_call_site!("Error listing columns for SQLite table {:?}: {}", table, err);
    });

    let mut column_types = Vec::<ItemType>::new();
    let mut column_defns = Vec::<Item>::new();

    for column in &columns {
        let column_decl = decl.as_mut()
            .and_then(|decl| decl.column_type_aliases.remove(&column.name));

        let column_ident = Ident::new_raw(&column.name, Span::mixed_site());

        let sql_ty = generate_column_sql_type(&column);

        let rust_ty: Box<Type> = column_decl.as_ref()
            .map(|decl| decl.ty.clone())
            .unwrap_or_else(|| parse_quote!{
                <#sql_ty as ::sqlitemapper::types::SqlType>::OwnedRustType
            });

        column_defns.push(Item::Struct(parse_quote! {
            pub struct #column_ident(::core::marker::PhantomData<()>);
        }));

        column_defns.push(Item::Impl(parse_quote! {
            impl ::sqlitemapper::types::Column for #column_ident {
                type SqlType = #sql_ty;
                type DomainType = #rust_ty;
            }
        }));

        column_types.push(parse_quote! {
            pub type #column_ident = #rust_ty;
        })
    }

    let column_types = token_stream(column_types);
    let column_defns = token_stream(column_defns);

    let table_name_span = decl.as_ref()
        .map(|decl| decl.ident.__span())
        .unwrap_or(Span::call_site());

    let table = Ident::new_raw(table, table_name_span);

    let unknown_items = decl.iter()
        .flat_map(|decl| &decl.unknown_items)
        .map(|item| item.to_token_stream())
        .collect::<TokenStream2>();

    let record_structs = generate_record_structs(&columns);

    parse_quote! {
        pub mod #table {
            pub mod columns {
                #column_defns
            }
            #record_structs
            #column_types
            #unknown_items
        }
    }
}

fn generate_column_sql_type(column: &TableColumn) -> Box<Type> {
    let inherent_type = match column.type_.as_str() {
        | "INT"
        | "INTEGER" => quote! { ::sqlitemapper::types::sql::Integer },
        | "REAL"    => quote! { ::sqlitemapper::types::sql::Real },
        | "TEXT"    => quote! { ::sqlitemapper::types::sql::Text },
        | "BLOB"    => quote! { ::sqlitemapper::types::sql::Blob },
        _ => { abort_call_site!("unknown sqlite datatype: {}", column.type_); }
    };

    let type_ = match column.not_null {
        true => inherent_type,
        false => quote! { ::sqlitemapper::types::sql::Nullable<#inherent_type> },
    };

    parse_quote! { #type_ }
}

fn get_single_primary_key(columns: &[TableColumn]) -> Option<&TableColumn> {
    let mut columns = columns.iter()
        .filter(|col| col.primary_key_part.is_some());

    let single = columns.next()?;
    match columns.next() {
        None => { return Some(single); }
        Some(_) => {return None; }
    }
}

fn primary_key_auto_assignable(columns: &[TableColumn]) -> bool {
    let Some(pkey) = get_single_primary_key(columns) else {
        return false;
    };

    if pkey.type_ == "INTEGER" {
        // alias for rowid
        return true;
    }

    if pkey.has_default {
        return true;
    }

    false
}

fn generate_record_structs(columns: &[TableColumn]) -> TokenStream2 {
    let fields = columns.iter()
        .map(generate_record_field)
        .collect::<TokenStream2>();

    let record_struct = quote! {
        pub struct Record {
            #fields
        }
    };

    let new_record_struct = if primary_key_auto_assignable(columns) {
        let fields = columns.iter()
            .filter(|col| col.primary_key_part.is_none())
            .map(generate_record_field)
            .collect::<TokenStream2>();

        quote! {
            pub struct NewRecord {
                #fields
            }
        }
    } else {
        quote!{}
    };

    quote!{
        #record_struct
        #new_record_struct
    }
}

fn generate_record_field(column: &TableColumn) -> TokenStream2 {
    let ident = Ident::new_raw(&column.name, Span::call_site());
    quote! { pub #ident: <columns::#ident as ::sqlitemapper::types::Column>::DomainType, }
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
