use std::marker::PhantomData;

use rusqlite::{Row, Error, types::ValueRef};

use crate::types::{SqlType, ConvertFromSqlType, ColumnList, Column, ColumnCons};

pub struct RowReader<'a, List> {
    row: &'a Row<'a>,
    idx: usize,
    _phantom: PhantomData<List>,
}

impl<'a, List: ColumnList> RowReader<'a, List> {
    pub(crate) fn new(row: &'a Row) -> Self {
        RowReader { row, idx: 0, _phantom: PhantomData }
    }

    pub fn column_index(&self) -> usize  {
        self.idx
    }

    pub fn column_name(&self) -> &str {
        // column idx is guaranteed to be within bounds:
        self.row.as_ref().column_name(self.idx).unwrap()
    }

    fn value_ref(&self) -> ValueRef<'a> {
        // column idx is guaranteed to be within bounds:
        self.row.get_ref(self.idx).unwrap()
    }

    #[cold]
    pub fn make_invalid_type_error(&self) -> Error {
        Error::InvalidColumnType(
            self.column_index(),
            self.column_name().to_owned(),
            self.value_ref().data_type(),
        )
    }

    pub fn get_integer(&self) -> Result<i64, Error> {
        self.row.get(self.idx)
    }

    pub fn get_real(&self) -> Result<f64, Error> {
        self.row.get(self.idx)
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

/*
impl<'a, Col: Column, Tail: ColumnList> RowReader<'a, ColumnCons<Nullable<Head>, Tail>> {
    /// only call this after checking `is_null`. There's no memory safety
    /// issues but it will cause errors
    pub(crate) fn as_nullable_interior(&self) -> RowReader<'a, SqlTypeCons<Head, Tail>> {
        RowReader { row: self.row, idx: self.idx, _phantom: PhantomData }
    }
}
*/

impl<'a, Col: Column, Tail: ColumnList> RowReader<'a, ColumnCons<Col, Tail>> {
    pub fn advance(&self) -> RowReader<'a, Tail> {
        RowReader {
            row: self.row,
            idx: self.idx + 1,
            _phantom: PhantomData,
        }
    }

    pub fn next(self) -> Result<(Col::RustType, RowReader<'a, Tail>), Error> {
        let value = Col::SqlType::get(self.value_ref())
            .map_err(|_| self.make_invalid_type_error())?;

        let value = Col::RustType::convert_from_sql_type(value)
            .map_err(|e| e.into_rusqlite_error(self.column_index()))?;

        Ok((value, self.advance()))
    }
}

pub trait FromRow<Row: ColumnList>: Sized {
    fn from_row<'a>(reader: RowReader<'a, Row>) -> Result<(Self, RowReader<'a, ()>), Error>;
}

impl FromRow<()> for () {
    fn from_row<'a>(reader: RowReader<'a, ()>) -> Result<(Self, RowReader<'a, ()>), Error> {
        Ok(((), reader))
    }
}

macro_rules! __make_column_cons {
    ( ( $col:ident, $($rest:ident,)* ) ) => {
        ColumnCons< $col, __make_column_cons!{ ( $($rest,)* ) } >
    };
    ( () ) => { () };
}

macro_rules! impl_from_row_for_tuple {
    { ( $( $nam:ident: $typ:ident, )* ) } => {
        impl < $( $typ: Column, )* >
            FromRow< __make_column_cons! { ( $( $typ, )* ) } >
            for ( $( $typ::RustType, )* )
        {
            fn from_row<'a>(
                reader: RowReader<'a, __make_column_cons! { ( $( $typ, )* ) } >
            ) -> Result<(Self, RowReader<'a, ()>), Error> {

                $(
                    let ($nam, reader) = reader.next()?;
                )*

                Ok(( ( $($nam ,)* ) , reader))
            }
        }
    };
}

impl_from_row_for_tuple!{ (t1: C1,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8, t9: C9,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8, t9: C9, t10: C10,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8, t9: C9, t10: C10, t11: C11,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8, t9: C9, t10: C10, t11: C11, t12: C12,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8, t9: C9, t10: C10, t11: C11, t12: C12, t13: C13,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8, t9: C9, t10: C10, t11: C11, t12: C12, t13: C13, t14: C14,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8, t9: C9, t10: C10, t11: C11, t12: C12, t13: C13, t14: C14, t15: C15,) }
impl_from_row_for_tuple!{ (t1: C1, t2: C2, t3: C3, t4: C4, t5: C5, t6: C6, t7: C7, t8: C8, t9: C9, t10: C10, t11: C11, t12: C12, t13: C13, t14: C14, t15: C15, t16: C16,) }
