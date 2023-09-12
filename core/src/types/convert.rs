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

    pub fn into_rusqlite_error(self, column_index: usize) -> rusqlite::Error {
        rusqlite::Error::FromSqlConversionFailure(
            column_index,
            self.data_type.clone(),
            Box::new(self),
        )
    }
}

pub trait ConvertFromSqlType<SqlT: SqlType>: Sized {
    fn convert_from_sql_type<'a>(value: SqlT::RustType<'a>) -> Result<Self, ConversionError>;
}

impl<T> ConvertFromSqlType<Integer> for T
    where T: TryFrom<i64>, T::Error: Debug + Send + Sync + 'static
{
    fn convert_from_sql_type(value: i64) -> Result<Self, ConversionError> {
        value.try_into().map_err(|err|
            ConversionError::new::<T, _>(Type::Integer, err))
    }
}

impl<T> ConvertFromSqlType<Real> for T
    where T: TryFrom<f64>, T::Error: Debug + Send + Sync + 'static
{
    fn convert_from_sql_type(value: f64) -> Result<Self, ConversionError> {
        value.try_into().map_err(|err|
            ConversionError::new::<T, _>(Type::Real, err))
    }
}

impl<T> ConvertFromSqlType<Text> for T
    where T: FromStr, T::Err: Debug + Send + Sync + 'static
{
    fn convert_from_sql_type<'a>(value: &'a str) -> Result<Self, ConversionError> {
        value.parse().map_err(|err|
            ConversionError::new::<T, _>(Type::Text, err))
    }
}

impl<T> ConvertFromSqlType<Blob> for T
    where
        for<'a> T: TryFrom<&'a [u8]>,
        for<'a> <T as TryFrom<&'a [u8]>>::Error: Debug + Send + Sync + 'static,
{
    fn convert_from_sql_type<'a>(value: &'a [u8]) -> Result<Self, ConversionError> {
        value.try_into().map_err(|err|
            ConversionError::new::<T, _>(Type::Blob, err))
    }
}

impl<Inner: SqlType, T> ConvertFromSqlType<Nullable<Inner>> for Option<T>
    where for<'a> T: ConvertFromSqlType<Inner>
{
    fn convert_from_sql_type<'a>(value: Option<Inner::RustType<'a>>) -> Result<Option<T>, ConversionError> {
        match value {
            None => Ok(None),
            Some(inner) => T::convert_from_sql_type(inner).map(Some),
        }
    }
}
