use std::marker::PhantomData;

use rusqlite::Error;
use rusqlite::types::{ValueRef, FromSqlError, Type, FromSqlResult};

use crate::from_row::RowReader;
use crate::types::SqlTypeList;

use super::ConvertFromSqlType;

pub trait SqlType: Sized {
    type RustType<'a>;

    fn read_from_row<'a, T: ConvertFromSqlType<Self>, Tail: SqlTypeList>(
        reader: RowReader<'a, (Self, Tail)>,
    ) -> (Result<T, Error>, RowReader<'a, Tail>);
}

pub struct Integer(PhantomData<()>);

impl SqlType for Integer {
    type RustType<'a> = i64;

    fn read_from_row<'a, T: ConvertFromSqlType<Integer>, Tail: SqlTypeList>(
        reader: RowReader<'a, (Self, Tail)>,
    ) -> (Result<T, Error>, RowReader<'a, Tail>) {
        let value = reader.get_integer()
            .and_then(|value| {
                T::convert_from_sql_type(value)
                    .map_err(|e| e.into_rusqlite_error(reader.column_index()))
            });

        (value, reader.advance())
    }
}

pub struct Real(PhantomData<()>);

impl SqlType for Real {
    type RustType<'a> = f64;

    fn read_from_row<'a, T: ConvertFromSqlType<Real>, Tail: SqlTypeList>(
        reader: RowReader<'a, (Self, Tail)>,
    ) -> (Result<T, Error>, RowReader<'a, Tail>) {
        let value = reader.get_real()
            .and_then(|value| {
                T::convert_from_sql_type(value)
                    .map_err(|e| e.into_rusqlite_error(reader.column_index()))
            });

        (value, reader.advance())
    }
}

pub struct Text(PhantomData<()>);

impl SqlType for Text {
    type RustType<'a> = &'a str;

    fn read_from_row<'a, T: ConvertFromSqlType<Text>, Tail: SqlTypeList>(
        reader: RowReader<'a, (Self, Tail)>,
    ) -> (Result<T, Error>, RowReader<'a, Tail>) {
        let value = reader.get_text()
            .and_then(|value| {
                T::convert_from_sql_type(value)
                    .map_err(|e| e.into_rusqlite_error(reader.column_index()))
            });

        (value, reader.advance())
    }
}

pub struct Blob(PhantomData<()>);

impl SqlType for Blob {
    type RustType<'a> = &'a [u8];

    fn read_from_row<'a, T: ConvertFromSqlType<Blob>, Tail: SqlTypeList>(
        reader: RowReader<'a, (Self, Tail)>,
    ) -> (Result<T, Error>, RowReader<'a, Tail>) {
        let value = reader.get_blob()
            .and_then(|value| {
                T::convert_from_sql_type(value)
                    .map_err(|e| e.into_rusqlite_error(reader.column_index()))
            });

        (value, reader.advance())
    }
}

pub struct Nullable<T: SqlType>(PhantomData<T>);

impl<Inner: SqlType> SqlType for Nullable<Inner> {
    type RustType<'a> = Option<Inner::RustType<'a>>;

    fn read_from_row<'a, T: ConvertFromSqlType<Nullable<Inner>>, Tail: SqlTypeList>(
        reader: RowReader<'a, (Self, Tail)>,
    ) -> (Result<T, Error>, RowReader<'a, Tail>) {
        todo!();
    }
}
