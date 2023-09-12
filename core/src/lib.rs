pub mod types;

pub use sqlitemapper_macros::{query, schema};
use types::{SqlType, SqlTypeList};

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

    // pub fn bind<P: Params>(&self, params: P) -> BoundQuery<SqlType, P> {
    //     BoundQuery { query: *self, params }
    // }
}

// pub struct BoundQuery<SqlType, ParamsType> {
//     query: Query<SqlType>,
//     params: ParamsType,
// }

// impl<RowType: FromRow, ParamsType: Params> BoundQuery<RowType, ParamsType> {
//     pub fn query_all(self, conn: &mut Connection) -> Result<Vec<RowType>, Error> {
//         conn.prepare(self.query.sql)?
//             .query(self.params)?
//             .mapped(RowType::from_row)
//             .collect()
//     }
// }
