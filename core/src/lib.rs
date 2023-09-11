mod from_row;
pub use from_row::FromRow;

use std::marker::PhantomData;

use rusqlite::{Params, Connection, Error};
pub use sqlitemapper_macros::{__query, schema};

pub struct Query<RowType> {
    sql: &'static str,
    _phantom: PhantomData<RowType>,
}

impl<RowType> Clone for Query<RowType> {
    fn clone(&self) -> Self {
        Query { sql: self.sql, _phantom: PhantomData }
    }
}

impl<RowType> Copy for Query<RowType> {}

impl<RowType: FromRow> Query<RowType> {
    pub fn new_unchecked(sql: &'static str) -> Self {
        Query {
            sql,
            _phantom: PhantomData,
        }
    }

    pub fn bind<P: Params>(&self, params: P) -> BoundQuery<RowType, P> {
        BoundQuery { query: *self, params }
    }
}

pub struct BoundQuery<RowType, ParamsType> {
    query: Query<RowType>,
    params: ParamsType,
}

impl<RowType: FromRow, ParamsType: Params> BoundQuery<RowType, ParamsType> {
    pub fn query_all(self, conn: &mut Connection) -> Result<Vec<RowType>, Error> {
        conn.prepare(self.query.sql)?
            .query(self.params)?
            .mapped(RowType::from_row)
            .collect()
    }
}
