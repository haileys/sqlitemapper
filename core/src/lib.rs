pub mod types;

use from_row::{FromRow, RowReader};
pub use sqlitemapper_macros::{query, schema};
use types::ColumnList;

mod from_row;

use std::marker::PhantomData;

use rusqlite::{Params, Connection, Error};

pub struct Query<Row> {
    sql: &'static str,
    _phantom: PhantomData<Row>,
}

impl<Row> Clone for Query<Row> {
    fn clone(&self) -> Self {
        Query { sql: self.sql, _phantom: PhantomData }
    }
}

impl<Row> Copy for Query<Row> {}

impl<Row: ColumnList> Query<Row> {
    pub fn new_unchecked(sql: &'static str) -> Self {
        Query {
            sql,
            _phantom: PhantomData,
        }
    }

    pub fn bind<P: Params>(&self, params: P) -> BoundQuery<Row, P> {
        BoundQuery { query: *self, params }
    }
}

pub struct BoundQuery<Row, P> {
    query: Query<Row>,
    params: P,
}

impl<Row: ColumnList, P: Params> BoundQuery<Row, P> {
    pub fn query_all<T: FromRow<Row>>(self, conn: &mut Connection) -> Result<Vec<T>, Error> {
        conn.prepare(self.query.sql)?
            .query_map(self.params, |row| {
                let reader = RowReader::<Row>::new(row);
                let (row, _) = T::from_row(reader)?;
                Ok(row)
            })?
            .collect::<Result<Vec<_>, _>>()
    }
}
