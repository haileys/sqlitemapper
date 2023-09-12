use std::marker::PhantomData;

use crate::types::SqlType;

pub struct SqlTypeCons<Head: SqlType, Tail: SqlTypeList>(PhantomData<(Head, Tail)>);

pub trait SqlTypeList {
    const N: usize;
}

impl SqlTypeList for () {
    const N: usize = 0;
}

impl<Head: SqlType, Tail: SqlTypeList> SqlTypeList for SqlTypeCons<Head, Tail> {
    const N: usize = Tail::N + 1;
}

pub trait SqlTypeListHead<Head> {
    type Tail: SqlTypeList;
}

impl<Head: SqlType, Tail: SqlTypeList> SqlTypeListHead<Head> for SqlTypeCons<Head, Tail> {
    type Tail = Tail;
}
