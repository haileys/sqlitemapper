pub mod types;

use from_row::{FromRow, RowReader};
pub use sqlitemapper_macros::{query, schema};
use types::SqlTypeList;

mod from_row;

use std::marker::PhantomData;

use rusqlite::{Params, Connection, Error};

pub struct Query<SqlRow> {
    sql: &'static str,
    _phantom: PhantomData<SqlRow>,
}

impl<SqlRow> Clone for Query<SqlRow> {
    fn clone(&self) -> Self {
        Query { sql: self.sql, _phantom: PhantomData }
    }
}

impl<SqlRow> Copy for Query<SqlRow> {}

impl<SqlRow: SqlTypeList> Query<SqlRow> {
    pub fn new_unchecked(sql: &'static str) -> Self {
        Query {
            sql,
            _phantom: PhantomData,
        }
    }

    pub fn bind<P: Params>(&self, params: P) -> BoundQuery<SqlRow, P> {
        BoundQuery { query: *self, params }
    }
}

pub struct BoundQuery<SqlRow, ParamsType> {
    query: Query<SqlRow>,
    params: ParamsType,
}

impl<SqlRow: SqlTypeList, ParamsType: Params> BoundQuery<SqlRow, ParamsType> {
    pub fn query_all<Row: FromRow<SqlRow>>(self, conn: &mut Connection) -> Result<Vec<Row>, Error> {
        conn.prepare(self.query.sql)?
            .query_map(self.params, |row| {
                let reader = RowReader::<SqlRow>::new(row);
                let (row, _) = Row::from_row(reader);
                row
            })?
            .collect::<Result<Vec<_>, _>>()
    }
}
