#![cfg(not(target_os = "android"))]

//! Module for error handling of the Lofelt SDK Core library.
//! A C-string conversion of the last error's message can be accessed via get_error_message().

use std::{
    cell::RefCell,
    os::raw::{c_char, c_int},
    ptr, slice,
};

pub const SUCCESS: c_int = 0;
pub const ERROR: c_int = -1;

/// The clip version is newer than the SDK version, and therefore some playback
/// features may not work.
pub const PARTIAL_VERSION_SUPPORT: c_int = 1;

thread_local! {
    // The last error that was passed into set__error().
    //
    // It's a thread local, so errors on one thread won't be accessible from another: this implies
    // that if the error message is important and needs to be logged, then it should be accessed in
    // the same thread as it was set.
    //
    // Q. Why is a RefCell used here?
    // A. Static values in Rust are immutable, so RefCell is used to provide 'interior mutability'.
    //    See: https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

// Caches the last error message encountered by the C API so that it can be inspected further.
pub fn set_error(error: String) -> c_int {
    LAST_ERROR.with(|last_error| {
        *last_error.borrow_mut() = Some(error);
    });
    ERROR
}

// Returns the size of the buffer required by get_error_message().
pub fn get_error_message_length() -> c_int {
    LAST_ERROR.with(|last_error| match last_error.borrow().as_ref() {
        // Rust strings aren't null-terminated, so +1 here for the null terminator
        // Rust's String type returns the number of bytes for len(), so this will be correct for
        // UTF-8 data.
        Some(error) => error.len() as c_int + 1,
        None => 0,
    })
}

// Writes a C-string conversion of the last error message to the provided buffer.
//
// To ensure that the buffer is large enough, the client can call get_error_message_length().
pub unsafe fn get_error_message(buffer: *mut c_char, length: c_int) -> c_int {
    if buffer.is_null() {
        return ERROR;
    }

    LAST_ERROR.with(|last_error| match last_error.borrow().as_ref() {
        Some(message) => {
            let buffer = slice::from_raw_parts_mut(buffer as *mut u8, length as usize);

            if message.len() >= buffer.len() {
                return ERROR;
            }

            ptr::copy_nonoverlapping(message.as_ptr(), buffer.as_mut_ptr(), message.len());

            // Rust strings aren't null-terminated, so append a null terminator now
            buffer[message.len()] = 0;

            SUCCESS
        }
        None => {
            // there was no error message, return an empty string
            let buffer = slice::from_raw_parts_mut(buffer as *mut u8, length as usize);

            //in case the given buffer is not null-terminated, append a null terminator now
            buffer[0] = 0;

            SUCCESS
        }
    })
}
