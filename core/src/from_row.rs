use std::{marker::PhantomData, str::FromStr};
use std::fmt::Debug;

use rusqlite::{Row, Error, types::Type};
use thiserror::Error;

use crate::types::{SqlTypeList, SqlType};

pub struct RowReader<'a, List> {
    row: &'a Row<'a>,
    idx: usize,
    _phantom: PhantomData<List>,
}

impl<'a, List: SqlTypeList> RowReader<'a, List> {
    pub fn new(row: &'a Row) -> Self {
        RowReader { row, idx: 0, _phantom: PhantomData }
    }
}

#[derive(Error, Debug)]
#[error("sqlitemapper type conversion error: column index={index}, data_type={data_type}: {error:?}")]
struct ConversionError<E: Debug> {
    index: usize,
    data_type: Type,
    error: E,
}

impl<E: Debug + Send + Sync + 'static> Into<rusqlite::Error> for ConversionError<E> {
    fn into(self) -> rusqlite::Error {
        Error::FromSqlConversionFailure(
            self.index,
            self.data_type.clone(),
            Box::new(self)
        )
    }
}

impl<'a, Head: SqlType, Tail: SqlTypeList> RowReader<'a, (Head, Tail)> {
    fn conversion_error<E>(&self, error: E) -> ConversionError<E>
        where E: Debug + Send + Sync + 'static
    {
        ConversionError {
            index: self.idx,
            data_type: self.row.get_ref(self.idx).unwrap().data_type(),
            error,
        }
    }

    fn next_reader(&self) -> RowReader<'a, Tail> {
        RowReader {
            row: self.row,
            idx: self.idx + 1,
            _phantom: PhantomData,
        }
    }
}

impl<'a, Tail: SqlTypeList> RowReader<'a, (i64, Tail)> {
    fn next<T>(self) -> (Result<T, Error>, RowReader<'a, Tail>)
        where T: TryFrom<i64>, T::Error: Debug + Send + Sync + 'static
    {
        let result = self.row.get::<_, i64>(self.idx)
            .and_then(|sql_value| {
                T::try_from(sql_value).map_err(|e| {
                    self.conversion_error(e).into()
                })
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
                T::try_from(sql_value).map_err(|e| {
                    self.conversion_error(e).into()
                })
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
                T::from_str(sql_str).map_err(|e| {
                    self.conversion_error(e).into()
                })
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
            .and_then(|sql_value| {
                T::try_from(sql_value).map_err(|e| {
                    self.conversion_error(e).into()
                })
            });

        (result, self.next_reader())
    }
}

pub trait ConvertFromSqlType<SqlT: SqlType> {}

impl<T> ConvertFromSqlType<i64> for T
    where T: TryFrom<i64>, T::Error: Debug + Send + Sync + 'static {}

impl<T> ConvertFromSqlType<f64> for T
    where T: TryFrom<f64>, T::Error: Debug + Send + Sync + 'static {}

impl<'a, T> ConvertFromSqlType<&'a str> for T
    where T: FromStr, T::Err: Debug + Send + Sync + 'static {}

impl<'a, T> ConvertFromSqlType<&'a [u8]> for T
    where T: TryFrom<&'a [u8]>, T::Error: Debug + Send + Sync + 'static {}

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
