use std::marker::PhantomData;

use super::{SqlType, FromSql};

pub struct ColumnCons<C: Column, Tail: ColumnList>(PhantomData<(C, Tail)>);

pub trait ColumnList {
    const N: usize;
}

impl ColumnList for () {
    const N: usize = 0;
}

impl<C: Column, Tail: ColumnList> ColumnList for ColumnCons<C, Tail> {
    const N: usize = 1 + Tail::N;
}

pub trait Column: Sized {
    type SqlType: SqlType;
    type DomainType: FromSql<Self::SqlType>;
}
