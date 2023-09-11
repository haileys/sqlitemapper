use std::ffi::CStr;
use std::ffi::c_char;
use std::ffi::c_int;
use std::marker::PhantomData;
use std::ptr;

use rusqlite::Connection;
use rusqlite::ffi as ffi;

pub struct Statement<'a> {
    raw: RawStatement<'a>,
}

fn to_int(u: usize) -> c_int {
    c_int::try_from(u).expect("usize -> c_int")
}

fn from_int(u: c_int) -> usize {
    usize::try_from(u).expect("c_int -> usize")
}

unsafe fn maybe_cstr<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr == ptr::null() {
        return None;
    }

    let cstr = CStr::from_ptr(ptr);
    cstr.to_str().ok()
}

impl<'a> Statement<'a> {
    pub fn prepare(conn: &'a mut Connection, sql: &str) -> Result<Self, rusqlite::Error> {
        let mut stmt = RawStatement::null();

        unsafe {
            let db = conn.handle();

            let rc = ffi::sqlite3_prepare_v2(
                db,
                sql.as_ptr() as *const i8,
                to_int(sql.len()),
                &mut stmt.ptr,
                ptr::null_mut(),
            );

            if rc != ffi::SQLITE_OK {
                return Err(crate::ffi::error_from_handle(db, rc));
            }
        }

        Ok(Statement { raw: stmt })
    }

    pub fn column_count(&self) -> usize {
        unsafe {
            from_int(ffi::sqlite3_column_count(self.raw.ptr))
        }
    }

    pub fn column_database(&self, idx: usize) -> Option<&str> {
        unsafe {
            maybe_cstr(ffi::sqlite3_column_database_name(self.raw.ptr, to_int(idx)))
        }
    }

    pub fn column_table(&self, idx: usize) -> Option<&str> {
        unsafe {
            maybe_cstr(ffi::sqlite3_column_table_name(self.raw.ptr, to_int(idx)))
        }
    }

    pub fn column_origin(&self, idx: usize) -> Option<&str> {
        unsafe {
            maybe_cstr(ffi::sqlite3_column_origin_name(self.raw.ptr, to_int(idx)))
        }
    }
}

pub struct RawStatement<'a> {
    ptr: *mut ffi::sqlite3_stmt,
    _phantom: PhantomData<&'a rusqlite::Connection>,
}

impl<'a> RawStatement<'a> {
    pub fn null() -> Self {
        RawStatement { ptr: ptr::null_mut(), _phantom: PhantomData }
    }
}

impl<'a> Drop for RawStatement<'a> {
    fn drop(&mut self) {
        unsafe {
            // no-op if ptr is null
            ffi::sqlite3_finalize(self.ptr);
        }
    }
}
