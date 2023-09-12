use std::marker::PhantomData;

use rusqlite::{Row, Error, types::ValueRef};

use crate::types::{SqlTypeList, SqlType, ConvertFromSqlType, sql::Nullable};

pub struct RowReader<'a, List> {
    row: &'a Row<'a>,
    idx: usize,
    _phantom: PhantomData<List>,
}

impl<'a, List: SqlTypeList> RowReader<'a, List> {
    pub(crate) fn new(row: &'a Row) -> Self {
        RowReader { row, idx: 0, _phantom: PhantomData }
    }

    pub fn column_index(&self) -> usize  {
        self.idx
    }

    pub fn get_integer(&self) -> Result<i64, Error> {
        self.row.get(self.idx)
    }

    pub fn get_real(&self) -> Result<f64, Error> {
        self.row.get(self.idx)
    }

    fn value_ref(&self) -> ValueRef<'a> {
        self.row.get_ref(self.idx).unwrap()
    }

    pub fn get_text(&self) -> Result<&'a str, Error> {
        Ok(self.value_ref().as_str()?)
    }

    pub fn get_blob(&self) -> Result<&'a [u8], Error> {
        Ok(self.value_ref().as_blob()?)
    }

    pub fn is_null(&self) -> bool {
        let typ = self.value_ref().data_type();
        typ == rusqlite::types::Type::Null
    }
}

impl<'a, Head: SqlType, Tail: SqlTypeList> RowReader<'a, (Nullable<Head>, Tail)> {
    /// only call this after checking `is_null`. There's no memory safety
    /// issues but it will cause errors
    pub(crate) fn as_nullable_interior(&self) -> RowReader<'a, (Head, Tail)> {
        RowReader { row: self.row, idx: self.idx, _phantom: PhantomData }
    }
}

impl<'a, Head: SqlType, Tail: SqlTypeList> RowReader<'a, (Head, Tail)> {
    pub fn advance(&self) -> RowReader<'a, Tail> {
        RowReader {
            row: self.row,
            idx: self.idx + 1,
            _phantom: PhantomData,
        }
    }
}

pub trait FromRow<SqlRow: SqlTypeList>: Sized {
    fn from_row<'a>(reader: RowReader<'a, SqlRow>) -> (Result<Self, Error>, RowReader<'a, ()>);
}

impl FromRow<()> for () {
    fn from_row<'a>(reader: RowReader<'a, ()>) -> (Result<Self, Error>, RowReader<'a, ()>) {
        (Ok(()), reader)
    }
}

impl<SqlT, T, SqlTail, TTail> FromRow<(SqlT, SqlTail)> for (T, TTail)
    where
        SqlT: SqlType,
        SqlTail: SqlTypeList,
        T: ConvertFromSqlType<SqlT>,
        TTail: FromRow<SqlTail>,
{
    fn from_row<'a>(reader: RowReader<'a, (SqlT, SqlTail)>) -> (Result<(T, TTail), Error>, RowReader<'a, ()>) {
        let (head, reader) = SqlT::read_from_row::<T, _>(reader);
        let (tail, reader) = TTail::from_row(reader);
        let result = head.and_then(|head| tail.map(|tail| (head, tail)));
        (result, reader)
    }
}
