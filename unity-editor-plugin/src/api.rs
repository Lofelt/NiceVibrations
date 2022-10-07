// Copyright (c) Meta Platforms, Inc. and affiliates.

use crate::{algorithm, GamepadRumble};
use std::{cell::RefCell, ffi::CString, os::raw::c_char, ptr, slice};

// See the similar LAST_ERROR in c_errors.rs for details
thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = RefCell::new(None);
}

// All public exported API in here starts with the "nv_plugin_" prefix to avoid symbol clashes with
// other libraries.

/// Returns the last error that happened when calling nv_plugin_convert_haptic_to_gamepad_rumble().
///
/// When no error happened ever, null is returned.
///
/// The returned error string is UTF-8 encoded with a null terminator. The ownership is not
/// transferred, the caller must not free the memory. The returned pointer is only valid until
/// the next call to nv_plugin_convert_haptic_to_gamepad_rumble().
#[no_mangle]
pub extern "C" fn nv_plugin_get_last_error() -> *const c_char {
    LAST_ERROR.with(|last_error| match last_error.borrow().as_ref() {
        Some(last_error) => last_error.as_ptr(),
        None => std::ptr::null(),
    })
}

fn set_last_error(message: &str) {
    eprintln!("{}", message);
    LAST_ERROR.with(|last_error| {
        let message = match CString::new(message) {
            Ok(message) => message,
            Err(_) => Default::default(), // Null bytes inside `message`, shouldn't happen
        };
        *last_error.borrow_mut() = Some(message);
    });
}

/// Returns the length of the last error returned by nv_plugin_get_last_error().
///
/// The length is the amount of bytes, excluding the null terminator.
#[no_mangle]
pub extern "C" fn nv_plugin_get_last_error_length() -> usize {
    LAST_ERROR.with(|last_error| match last_error.borrow().as_ref() {
        Some(last_error) => last_error.as_bytes().len(),
        None => 0,
    })
}

/// Convert a haptic clip given as a JSON string to a GamepadRumble.
///
/// This is a wrapper around convert_haptic_to_gamepad_rumble_inner() that handles error results
/// in a way that the C# caller can deal with.
///
/// The method returns a GamepadRumble and passes the ownership to the caller. The caller needs
/// to call nv_plugin_destroy() to free the returned GamepadRumble once it is done using it.
///
/// Since the returned GamepadRumble can not be inspected by C# directly, C# can use
/// nv_plugin_get_durations(), nv_plugin_get_low_frequency_motor_speeds() and
/// nv_plugin_get_high_frequency_motor_speeds() to access the individual fields of GamepadRumble.
///
/// If an error occurs, null is returned, and the caller can check the error with
/// nv_plugin_get_last_error().
///
/// # Parameters
/// `data` is UTF-8 encoded, without a null terminator
///
/// # Safety
/// `data` needs to be a valid pointer to an array of bytes at least `data_size_bytes` bytes large
#[no_mangle]
pub extern "C" fn nv_plugin_convert_haptic_to_gamepad_rumble(
    data: *const c_char,
    data_size_bytes: usize,
) -> *mut GamepadRumble {
    let data = unsafe { slice::from_raw_parts(data as *const u8, data_size_bytes) };
    let gamepad_rumble = algorithm::convert_haptic_to_gamepad_rumble_inner(data);
    match gamepad_rumble {
        Ok(gamepad_rumble) => Box::into_raw(Box::new(gamepad_rumble)),
        Err(err) => {
            set_last_error(&err.message);
            std::ptr::null_mut()
        }
    }
}

/// Frees the GamepadRumble that is passed to it.
///
/// # Safety
/// `gamepad_rumble` needs to be a valid pointer to a GamepadRumble that was
/// created with nv_plugin_convert_haptic_to_gamepad_rumble().
#[no_mangle]
pub unsafe extern "C" fn nv_plugin_destroy(gamepad_rumble: *mut GamepadRumble) {
    Box::from_raw(gamepad_rumble);
}

/// Returns the number of entries in the GamepadRumble.
///
/// # Safety
/// `gamepad_rumble` needs to be a valid pointer to a GamepadRumble that was
/// created with nv_plugin_convert_haptic_to_gamepad_rumble().
#[no_mangle]
pub unsafe extern "C" fn nv_plugin_get_length(gamepad_rumble: *mut GamepadRumble) -> usize {
    (*gamepad_rumble).durations_ms.len()
}

/// Copies the durations array from the given GamepadRumble to `durations_ms_out`.
///
/// # Safety
/// - `gamepad_rumble` needs to be a valid pointer to a GamepadRumble that was
///   created with nv_plugin_convert_haptic_to_gamepad_rumble().
/// - `durations_ms_out` needs to be a valid writable pointer to an array that can
///   fit at least as many entries as GamepadRumble has.
#[no_mangle]
pub unsafe extern "C" fn nv_plugin_get_durations(
    gamepad_rumble: *const GamepadRumble,
    durations_ms_out: *mut i32,
) {
    ptr::copy(
        (*gamepad_rumble).durations_ms.as_ptr(),
        durations_ms_out,
        (*gamepad_rumble).durations_ms.len(),
    );
}

/// Copies the low frequency motor speeds array from the given GamepadRumble to
/// `low_frequencies_out`.
///
/// # Safety
/// - `gamepad_rumble` needs to be a valid pointer to a GamepadRumble that was
///   created with nv_plugin_convert_haptic_to_gamepad_rumble().
/// - `low_frequencies_out` needs to be a valid writable pointer to an array that can
///   fit at least as many entries as GamepadRumble has.
#[no_mangle]
pub unsafe extern "C" fn nv_plugin_get_low_frequency_motor_speeds(
    gamepad_rumble: *const GamepadRumble,
    low_frequencies_out: *mut f32,
) {
    ptr::copy(
        (*gamepad_rumble).low_frequency_motor_speeds.as_ptr(),
        low_frequencies_out,
        (*gamepad_rumble).low_frequency_motor_speeds.len(),
    );
}

/// Copies the high frequency motor speeds array from the given GamepadRumble to
/// `high_frequencies_out`.
///
/// # Safety
/// - `gamepad_rumble` needs to be a valid pointer to a GamepadRumble that was
///   created with nv_plugin_convert_haptic_to_gamepad_rumble().
/// - `high_frequencies_out` needs to be a valid writable pointer to an array that can
///   fit at least as many entries as GamepadRumble has.
#[no_mangle]
pub unsafe extern "C" fn nv_plugin_get_high_frequency_motor_speeds(
    gamepad_rumble: *const GamepadRumble,
    high_frequencies_out: *mut f32,
) {
    ptr::copy(
        (*gamepad_rumble).high_frequency_motor_speeds.as_ptr(),
        high_frequencies_out,
        (*gamepad_rumble).high_frequency_motor_speeds.len(),
    );
}
