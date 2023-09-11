pub use sqlitemapper_macros::{query, schema};

pub struct Query {
    pub sql: &'static str,
    pub columns: Vec<Column>,
}

pub struct Column {
    pub sql_type: Option<&'static str>,
}
