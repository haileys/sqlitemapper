use std::marker::PhantomData;

use rusqlite::{Row, Error, types::ValueRef};

use crate::types::{SqlTypeList, SqlType, ConvertFromSqlType, sql::Nullable, SqlTypeCons};

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

impl<'a, Head: SqlType, Tail: SqlTypeList> RowReader<'a, SqlTypeCons<Nullable<Head>, Tail>> {
    /// only call this after checking `is_null`. There's no memory safety
    /// issues but it will cause errors
    pub(crate) fn as_nullable_interior(&self) -> RowReader<'a, SqlTypeCons<Head, Tail>> {
        RowReader { row: self.row, idx: self.idx, _phantom: PhantomData }
    }
}

impl<'a, Head: SqlType, Tail: SqlTypeList> RowReader<'a, SqlTypeCons<Head, Tail>> {
    pub fn advance(&self) -> RowReader<'a, Tail> {
        RowReader {
            row: self.row,
            idx: self.idx + 1,
            _phantom: PhantomData,
        }
    }
}

pub trait FromRow<SqlRow: SqlTypeList>: Sized {
    fn from_row<'a>(reader: RowReader<'a, SqlRow>) -> Result<(Self, RowReader<'a, ()>), Error>;
}

impl FromRow<()> for () {
    fn from_row<'a>(reader: RowReader<'a, ()>) -> Result<(Self, RowReader<'a, ()>), Error> {
        Ok(((), reader))
    }
}

pub struct ValueCons<Head, Tail>(Head, Tail);

impl<SqlT, T, SqlTail, TTail> FromRow<SqlTypeCons<SqlT, SqlTail>> for ValueCons<T, TTail>
    where
        SqlT: SqlType,
        SqlTail: SqlTypeList,
        T: ConvertFromSqlType<SqlT>,
        TTail: FromRow<SqlTail>,
{
    fn from_row<'a>(reader: RowReader<'a, SqlTypeCons<SqlT, SqlTail>>) -> Result<(ValueCons<T, TTail>, RowReader<'a, ()>), Error> {
        let (head, reader) = SqlT::read_from_row::<T, _>(reader)?;
        let (tail, reader) = TTail::from_row(reader)?;
        Ok((ValueCons(head, tail), reader))
    }
}

macro_rules! __make_sql_type_cons {
    ( ( $head:ident, $($rest:ident,)* ) ) => {
        SqlTypeCons< $head, __make_sql_type_cons!{ ( $($rest,)* ) } >
    };
    ( () ) => { () };
}

macro_rules! impl_tuple_from_row {
    { $( $sql:ident => $var:ident : $out:ident, )* } => {
        impl < $( $sql, $out, )* >
            FromRow< __make_sql_type_cons! { ( $( $sql, )* ) } >
            for ( $( $out, )* )
        where
            $(
                $sql: SqlType,
                $out: for<'a> From<$sql::RustType<'a>>,
            )*
        {
            fn from_row<'a>(
                reader: RowReader<'a, __make_sql_type_cons! { ( $( $sql, )* ) } >
            ) -> Result<(Self, RowReader<'a, ()>), Error> {
                $(
                    let $var = $sql::map_value_from_row(&reader, |item| Ok($out::from(item)))?;
                    let reader = reader.advance();
                )*

                Ok(( ( $($var ,)* ) , reader))
            }
        }
    };
}

impl_tuple_from_row!{
    S1 => t1: T1,
}

impl_tuple_from_row!{
    S1 => t1: T1,
    S2 => t2: T2,
}

impl_tuple_from_row!{
    S1 => t1: T1,
    S2 => t2: T2,
    S3 => t3: T3,
}

impl_tuple_from_row!{
    S1 => t1: T1,
    S2 => t2: T2,
    S3 => t3: T3,
    S4 => t4: T4,
}

/*
impl<S1, T1> FromRow<(S1, ())> for (T1,)
    where
        S1: SqlType,
        T1: for<'a> From<S1::RustType<'a>>,
{
    fn from_row<'a>(reader: RowReader<'a, (S1, ())>) -> Result<(Self, RowReader<'a, ()>), Error> {
        let t1 = S1::map_value_from_row(&reader, |item| Ok(T1::from(item)))?;
        let reader = reader.advance();

        Ok(((t1,), reader))
    }
}
*/
