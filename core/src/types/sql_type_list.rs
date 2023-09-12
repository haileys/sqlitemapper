use crate::types::SqlType;

trait Sealed {}
#[allow(private_bounds)]
pub trait SqlTypeList: Sealed {
    const N: usize;
}

impl Sealed for () {}
impl SqlTypeList for () {
    const N: usize = 0;
}

impl<Head: SqlType, Tail: SqlTypeList> Sealed for (Head, Tail) {}
impl<Head: SqlType, Tail: SqlTypeList> SqlTypeList for (Head, Tail) {
    const N: usize = Tail::N + 1;
}

pub trait SqlTypeListHead<Head> {
    type Tail: SqlTypeList;
}

impl<Head: SqlType, Tail: SqlTypeList> SqlTypeListHead<Head> for (Head, Tail) {
    type Tail = Tail;
}
