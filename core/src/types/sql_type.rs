use std::marker::PhantomData;

use rusqlite::Error;

use crate::from_row::RowReader;
use crate::types::{SqlTypeList, SqlTypeCons, ConvertFromSqlType};

pub trait SqlType: Sized {
    type RustType<'a>;

    fn map_value_from_row<'a, R, Tail: SqlTypeList>(
        reader: &RowReader<'a, SqlTypeCons<Self, Tail>>,
        func: impl FnOnce(Self::RustType<'a>) -> Result<R, Error>,
    ) -> Result<R, Error>;

    fn read_from_row<'a, T: ConvertFromSqlType<Self>, Tail: SqlTypeList>(
        reader: RowReader<'a, SqlTypeCons<Self, Tail>>,
    ) -> Result<(T, RowReader<'a, Tail>), Error> {
        let value = Self::map_value_from_row(&reader, |value| {
            T::convert_from_sql_type(value)
                .map_err(|e| e.into_rusqlite_error(reader.column_index()))
        })?;

        Ok((value, reader.advance()))
    }
}

pub struct Integer(PhantomData<()>);

impl SqlType for Integer {
    type RustType<'a> = i64;

    fn map_value_from_row<'a, R, Tail: SqlTypeList>(
        reader: &RowReader<'a, SqlTypeCons<Self, Tail>>,
        func: impl FnOnce(Self::RustType<'a>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        reader.get_integer().and_then(func)
    }
}

pub struct Real(PhantomData<()>);

impl SqlType for Real {
    type RustType<'a> = f64;

    fn map_value_from_row<'a, R, Tail: SqlTypeList>(
        reader: &RowReader<'a, SqlTypeCons<Self, Tail>>,
        func: impl FnOnce(Self::RustType<'a>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        reader.get_real().and_then(func)
    }
}

pub struct Text(PhantomData<()>);

impl SqlType for Text {
    type RustType<'a> = &'a str;

    fn map_value_from_row<'a, R, Tail: SqlTypeList>(
        reader: &RowReader<'a, SqlTypeCons<Self, Tail>>,
        func: impl FnOnce(Self::RustType<'a>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        reader.get_text().and_then(func)
    }
}

pub struct Blob(PhantomData<()>);

impl SqlType for Blob {
    type RustType<'a> = &'a [u8];

    fn map_value_from_row<'a, R, Tail: SqlTypeList>(
        reader: &RowReader<'a, SqlTypeCons<Self, Tail>>,
        func: impl FnOnce(Self::RustType<'a>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        reader.get_blob().and_then(func)
    }
}

pub struct Nullable<T: SqlType>(PhantomData<T>);

impl<Inner: SqlType> SqlType for Nullable<Inner> {
    type RustType<'a> = Option<Inner::RustType<'a>>;

    fn map_value_from_row<'a, R, Tail: SqlTypeList>(
        reader: &RowReader<'a, SqlTypeCons<Self, Tail>>,
        func: impl FnOnce(Self::RustType<'a>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if reader.is_null() {
            func(None)
        } else {
            let reader = reader.as_nullable_interior();
            Inner::map_value_from_row(&reader, |value| func(Some(value)))
        }
    }
}
