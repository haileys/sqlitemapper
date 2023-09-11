mod statement;
mod ffi;

use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use rusqlite::types::Type;
use statement::Statement;
use thiserror::Error;

pub struct Schema {
    conn: Mutex<Connection>,
}

pub type SqlError = rusqlite::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("opening memory database: {0}")]
    Open(rusqlite::Error),
    #[error("executing schema sql: {0}")]
    Sql(rusqlite::Error),
    #[error("loading schema file: {0}")]
    Io(std::io::Error),
}

#[derive(Error, Debug)]
pub enum PrepareError {
    #[error("preparing query: {0}")]
    Sql(rusqlite::Error),
}

impl Schema {
    pub fn from_sql(sql: &str) -> Result<Self, LoadError> {
        let conn = Connection::open_in_memory()
            .map_err(LoadError::Open)?;

        conn.execute_batch(sql)
            .map_err(LoadError::Sql)?;

        let conn = Mutex::new(conn);

        Ok(Schema { conn })
    }

    pub fn from_file(path: &Path) -> Result<Self, LoadError> {
        let sql = std::fs::read_to_string(path)
            .map_err(LoadError::Io)?;

        Self::from_sql(&sql)
    }

    pub fn prepare(&self, sql: &str) -> Result<QueryInfo, PrepareError> {
        let mut conn = self.conn.lock().unwrap();

        let stmt = Statement::prepare(&mut conn, sql)
            .map_err(PrepareError::Sql)?;

        let mut columns = Vec::with_capacity(stmt.column_count());
        for i in 0..stmt.column_count() {
            let column = ResultColumn {
                index: i,
                name: stmt.name(i).map(|s| s.to_owned()),
                origin_database: stmt.origin_database(i).map(|s| s.to_owned()),
                origin_table: stmt.origin_table(i).map(|s| s.to_owned()),
                origin_column: stmt.origin_column(i).map(|s| s.to_owned()),
            };
            columns.push(column);
        }

        Ok(QueryInfo { columns })
    }

    pub fn tables(&self) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        struct Table {
            schema: String,
            name: String,
        }

        let tables = conn
            .prepare("PRAGMA table_list")?
            .query_map([], |row| {
                Ok(Table { schema: row.get(0)?, name: row.get(1)? })
            })?
            .filter_map(Result::ok)
            .filter(|t| t.schema == "main")
            .map(|t| t.name)
            .collect();

        Ok(tables)
    }

    pub fn columns(&self, table: &str) -> Result<Vec<TableColumn>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        let columns = conn
            .prepare(&format!("PRAGMA table_info({})", table))?
            .query_map([], |row| {
                Ok(TableColumn {
                    name: row.get(1)?,
                    type_: row.get(2)?,
                    not_null: row.get(3)?,
                    has_default: row.get_ref(4)?.data_type() == Type::Null,
                    primary_key_part: NonZeroUsize::new(row.get(5)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(columns)
    }
}

pub struct TableColumn {
    pub name: String,
    pub type_: String,
    pub not_null: bool,
    pub has_default: bool,
    pub primary_key_part: Option<NonZeroUsize>,
}

pub struct QueryInfo {
    columns: Vec<ResultColumn>
}

impl QueryInfo {
    pub fn columns(&self) -> &[ResultColumn] {
        &self.columns
    }
}

pub struct ResultColumn {
    index: usize,
    name: Option<String>,
    origin_database: Option<String>,
    origin_table: Option<String>,
    origin_column: Option<String>,
}

impl ResultColumn {
    pub fn describe(&self) -> String {
        if let Some(name) = &self.name {
            format!("Column '{name}'")
        } else {
            let idx1 = self.index + 1;
            let suffix = match idx1 % 10 {
                1 => "st",
                2 => "nd",
                3 => "rd",
                _ => "th",
            };
            format!("{idx1}{suffix} selected column in query")
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn origin_database(&self) -> Option<&str> {
        self.origin_database.as_deref()
    }

    pub fn origin_table(&self) -> Option<&str> {
        self.origin_table.as_deref()
    }

    pub fn origin_column(&self) -> Option<&str> {
        self.origin_column.as_deref()
    }
}
