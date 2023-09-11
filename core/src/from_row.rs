use rusqlite::{Row, Error, types::FromSql};

pub trait FromRow where Self: Sized {
    fn from_row(row: &Row) -> Result<Self, Error>;
}

struct RowReader<'a> {
    row: &'a Row<'a>,
    idx: usize,
}

impl<'a> RowReader<'a> {
    pub fn new(row: &'a Row) -> Self {
        RowReader { row, idx: 0 }
    }

    pub fn next<T: FromSql>(&mut self) -> Result<T, Error> {
        let result = self.row.get(self.idx);
        self.idx += 1;
        result
    }
}

macro_rules! impl_tuple_from_row {
    { ( $($t:ident,)* ) } => {
        impl<$($t: FromSql,)*> FromRow for ($($t,)*) {
            fn from_row(row: &Row) -> Result<Self, Error> {
                #[allow(unused_mut, unused_variables)]
                let mut row = RowReader::new(row);
                Ok((
                    $(row.next::<$t>()?,)*
                ))
            }
        }
    };
}

impl_tuple_from_row!{ () }
impl_tuple_from_row!{ (T0,) }
impl_tuple_from_row!{ (T0,T1,) }
impl_tuple_from_row!{ (T0,T1,T2,) }
impl_tuple_from_row!{ (T0,T1,T2,T3,) }
impl_tuple_from_row!{ (T0,T1,T2,T3,T4,) }
impl_tuple_from_row!{ (T0,T1,T2,T3,T4,T5,) }
impl_tuple_from_row!{ (T0,T1,T2,T3,T4,T5,T6,) }
impl_tuple_from_row!{ (T0,T1,T2,T3,T4,T5,T6,T7,) }
