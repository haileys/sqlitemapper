use std::ffi::{CStr, c_char, c_int};

use rusqlite::{Error, ffi};

pub unsafe fn errmsg_to_string(
    errmsg: *const c_char,
) -> String {
    let c_slice = CStr::from_ptr(errmsg).to_bytes();
    String::from_utf8_lossy(c_slice).into_owned()
}

pub unsafe fn error_from_handle(
    db: *mut ffi::sqlite3,
    code: c_int,
) -> Error {
    let message = if db.is_null() {
        None
    } else {
        Some(errmsg_to_string(ffi::sqlite3_errmsg(db)))
    };
    error_from_sqlite_code(code, message)
}

pub fn error_from_sqlite_code(code: c_int, message: Option<String>) -> Error {
    Error::SqliteFailure(ffi::Error::new(code), message)
}
