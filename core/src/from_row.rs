use std::marker::PhantomData;

use rusqlite::{Row, Error};

use crate::types::{SqlTypeList, SqlType, ConvertFromSqlType};

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

    pub fn get_text(&self) -> Result<&str, Error> {
        let value = self.row.get_ref(self.idx)?;
        let str = value.as_str()?;
        Ok(str)
    }

    pub fn get_blob(&self) -> Result<&[u8], Error> {
        let value = self.row.get_ref(self.idx)?;
        let blob = value.as_blob()?;
        Ok(blob)
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

/*
impl<'a, Tail: SqlTypeList> RowReader<'a, (i64, Tail)> {
    fn next<T>(self) -> (Result<T, Error>, RowReader<'a, Tail>)
        where T: ConvertFromSqlType<i64>
    {
        let result = self.row.get::<_, i64>(self.idx)
            .and_then(|sql_value| {
                T::convert_from_sql_type(sql_value)
                    .map_err(|e| e.into_rusqlite_error(self.idx))
            });

        (result, self.next_reader())
    }
}

impl<'a, Tail: SqlTypeList> RowReader<'a, (f64, Tail)> {
    fn next<T>(self) -> (Result<T, Error>, RowReader<'a, Tail>)
        where T: TryFrom<f64>, T::Error: Debug + Send + Sync + 'static
    {
        let result = self.row.get::<_, f64>(self.idx)
            .and_then(|sql_value| {
                T::convert_from_sql_type(sql_value)
                    .map_err(|e| e.into_rusqlite_error(self.idx))
            });

        (result, self.next_reader())
    }
}

impl<'a, Tail: SqlTypeList> RowReader<'a, (&'a str, Tail)> {
    fn next<T>(self) -> (Result<T, Error>, RowReader<'a, Tail>)
        where T: FromStr, T::Err: Debug + Send + Sync + 'static
    {
        let result = self.row.get_ref(self.idx)
            .and_then(|sql_value| sql_value.as_str().map_err(Into::into))
            .and_then(|sql_str| {
                T::convert_from_sql_type(sql_str)
                    .map_err(|e| e.into_rusqlite_error(self.idx))
            });

        (result, self.next_reader())
    }
}

impl<'a, Tail: SqlTypeList> RowReader<'a, (&'a [u8], Tail)> {
    fn next<T>(self) -> (Result<T, Error>, RowReader<'a, Tail>)
        where T: TryFrom<&'a [u8]>, T::Error: Debug + Send + Sync + 'static
    {
        let result = self.row.get_ref(self.idx)
            .and_then(|sql_value| sql_value.as_blob().map_err(Into::into))
            .and_then(|sql_blob| {
                T::convert_from_sql_type(sql_blob)
                    .map_err(|e| e.into_rusqlite_error(self.idx))
            });

        (result, self.next_reader())
    }
}
*/

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

/*
pub trait ConvertList {}

impl ConvertList for () {}

impl<SqlT: SqlType, UserT, Tail: ConvertList> ConvertList for (SqlT, UserT, Tail)
    where UserT: ConvertFromSqlType<SqlT> {}

pub trait TypeList {}
impl TypeList for () {}
impl<T, Tail: TypeList> TypeList for (T, Tail) {}

pub trait ZipTypeLists {
    type Output: ConvertList;
}
impl ZipTypeLists for ((), ()) {
    type Output = ();
}
impl<SqlT, SqlTail, UserT, UserTail> ZipTypeLists for ((SqlT, SqlTail), (UserT, UserTail))
    where
        SqlT: SqlType,
        SqlTail: SqlTypeList,
        UserT: ConvertFromSqlType<SqlT>,
        UserTail: TypeList,
        (SqlTail, UserTail): ZipTypeLists
{
    type Output = (SqlT, UserT, <(SqlTail, UserTail) as ZipTypeLists>::Output);
}

pub trait FromPartialRow<'a, List> where Self: Sized {
    type Error;
    type Tail: SqlTypeList;
    fn from_partial_row(row: RowReader<'a, List>) -> (Result<Self, Self::Error>, RowReader<'a, Self::Tail>);
}

impl<'a, Tail: SqlTypeList> FromPartialRow<'a, (i64, Tail)> for i64 {
    type Error = rusqlite::Error;
    type Tail = Tail;

    fn from_partial_row(row: RowReader<'a, (i64, Tail)>) -> (Result<Self, Self::Error>, RowReader<'a, Self::Tail>) {
        row.next()
    }
}

macro_rules! tuple_row_cons_list {
    ($head:ty,$($tail:ty,)*) => { ($t, tuple_row_cons_list!($($tail,)*)) };
    () => { () };
}

macro_rules! impl_tuple_row {
    ($($t:ty,)*) => {
        impl<'a, $($t: ,)*, Tail: SqlTypeList> FromPartialRow<'a>
    }
}
*/
