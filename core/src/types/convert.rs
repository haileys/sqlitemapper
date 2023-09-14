use std::fmt::Debug;
use std::str::FromStr;

use rusqlite::types::Type;
use thiserror::Error;

use crate::types::SqlType;
use crate::types::sql::{Integer, Real, Text, Blob, Nullable};

#[derive(Error, Debug)]
#[error("sqlitemapper type conversion error: from {data_type} to {target_type}: {error:?}")]
pub struct ConversionError {
    data_type: Type,
    target_type: &'static str,
    error: Box<dyn Debug + Send + Sync + 'static>,
}

impl ConversionError {
    pub fn new<T, E>(data_type: Type, error: E) -> Self
        where E: Debug + Send + Sync + 'static
    {
        let error = Box::new(error);
        let target_type = std::any::type_name::<T>();
        ConversionError { data_type, target_type, error }
    }

    #[cold]
    pub fn into_rusqlite_error(self, column_index: usize) -> rusqlite::Error {
        rusqlite::Error::FromSqlConversionFailure(
            column_index,
            self.data_type.clone(),
            Box::new(self),
        )
    }
}

pub trait FromSql<SqlT: SqlType>: Sized {
    fn from_sql<'a>(value: SqlT::RustType<'a>) -> Result<Self, ConversionError>;
}

impl<T> FromSql<Integer> for T
    where T: TryFrom<i64>, T::Error: Debug + Send + Sync + 'static
{
    fn from_sql(value: i64) -> Result<Self, ConversionError> {
        value.try_into().map_err(|err|
            ConversionError::new::<T, _>(Type::Integer, err))
    }
}

impl<T> FromSql<Real> for T
    where T: TryFrom<f64>, T::Error: Debug + Send + Sync + 'static
{
    fn from_sql(value: f64) -> Result<Self, ConversionError> {
        value.try_into().map_err(|err|
            ConversionError::new::<T, _>(Type::Real, err))
    }
}

impl<T> FromSql<Text> for T
    where T: FromStr, T::Err: Debug + Send + Sync + 'static
{
    fn from_sql<'a>(value: &'a str) -> Result<Self, ConversionError> {
        value.parse().map_err(|err|
            ConversionError::new::<T, _>(Type::Text, err))
    }
}

impl<T> FromSql<Blob> for T
    where
        for<'a> T: TryFrom<&'a [u8]>,
        for<'a> <T as TryFrom<&'a [u8]>>::Error: Debug + Send + Sync + 'static,
{
    fn from_sql<'a>(value: &'a [u8]) -> Result<Self, ConversionError> {
        value.try_into().map_err(|err|
            ConversionError::new::<T, _>(Type::Blob, err))
    }
}

impl<Inner: SqlType, T: FromSql<Inner>> FromSql<Nullable<Inner>> for Option<T>
{
    fn from_sql<'a>(value: Option<Inner::RustType<'a>>) -> Result<Option<T>, ConversionError> {
        match value {
            None => Ok(None),
            Some(inner) => T::from_sql(inner).map(Some),
        }
    }
}
