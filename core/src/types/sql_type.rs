use rusqlite::types::{ValueRef, FromSqlError, Type};

trait Sealed {}

#[allow(private_bounds)]
pub trait SqlType: Sealed + Sized {}

impl Sealed for i64 {}
impl SqlType for i64 {}

impl Sealed for f64 {}
impl SqlType for f64 {}

impl<'s> Sealed for &'s str {}
impl<'s> SqlType for &'s str {}

impl<'s> Sealed for &'s [u8] {}
impl<'s> SqlType for &'s [u8] {}

impl<T: Sealed + SqlType> Sealed for Option<T> {}
impl<T: Sealed + SqlType> SqlType for Option<T> {}
