use std::marker::PhantomData;

use rusqlite::types::{ValueRef, FromSqlResult, Type};

pub trait SqlType: Sized {
    type RustType<'a>;
    type OwnedRustType;

    fn get<'a>(value: ValueRef<'a>) -> FromSqlResult<Self::RustType<'a>>;
}

pub struct Integer(PhantomData<()>);

impl SqlType for Integer {
    type RustType<'a> = i64;
    type OwnedRustType = i64;

    fn get<'a>(value: ValueRef<'a>) -> FromSqlResult<Self::RustType<'a>> {
        value.as_i64()
    }
}

pub struct Real(PhantomData<()>);

impl SqlType for Real {
    type RustType<'a> = f64;
    type OwnedRustType = f64;

    fn get<'a>(value: ValueRef<'a>) -> FromSqlResult<Self::RustType<'a>> {
        value.as_f64()
    }
}

pub struct Text(PhantomData<()>);

impl SqlType for Text {
    type RustType<'a> = &'a str;
    type OwnedRustType = String;

    fn get<'a>(value: ValueRef<'a>) -> FromSqlResult<Self::RustType<'a>> {
        value.as_str()
    }
}

pub struct Blob(PhantomData<()>);

impl SqlType for Blob {
    type RustType<'a> = &'a [u8];
    type OwnedRustType = Vec<u8>;

    fn get<'a>(value: ValueRef<'a>) -> FromSqlResult<Self::RustType<'a>> {
        value.as_blob()
    }
}

pub struct Nullable<T: SqlType>(PhantomData<T>);

impl<Inner: SqlType> SqlType for Nullable<Inner> {
    type RustType<'a> = Option<Inner::RustType<'a>>;
    type OwnedRustType = Option<Inner::OwnedRustType>;

    fn get<'a>(value: ValueRef<'a>) -> FromSqlResult<Self::RustType<'a>> {
        if value.data_type() == Type::Null {
            Ok(None)
        } else {
            Ok(Some(Inner::get(value)?))
        }
    }
}
