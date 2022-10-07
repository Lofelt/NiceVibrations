// Copyright (c) Meta Platforms, Inc. and affiliates.

#![cfg(not(target_os = "android"))]

//! The functions are exposed in a way so they can be called as a C API
//!
//! # Safety
//! All functions taking a `*mut LofeltSdkController` as an argument are unsafe. To be safe, the
//! argument needs to be a valid pointer to a `HapticsController`, created with
//! `lofelt_sdk_controller_create()`.
//!
//! # Error Handling
//! All public functions return a `c_int` to indicate the error status. This is equal to `ERROR`
//! if the operation failed. In that case, `lofelt_sdk_get_error_message()` can be called to get
//! additional information about the error.

use {
    crate::c_errors::{
        get_error_message, get_error_message_length, set_error, PARTIAL_VERSION_SUPPORT, SUCCESS,
    },
    env_logger::Builder,
    lib::{
        clip_players::{
            self,
            streaming::{self, AmplitudeEvent, FrequencyEvent},
        },
        HapticsController, VersionSupport,
    },
    std::{
        ffi::c_void,
        io::Write,
        os::raw::{c_char, c_float, c_int},
        slice,
        sync::Once,
    },
};

struct CVoidPtr(*mut c_void);
unsafe impl Send for CVoidPtr {}
unsafe impl Sync for CVoidPtr {}

// Publicly-facing struct wrapping `lib::HapticsController`.
// This allows client code of the generated C API to maintain a handle to an instance of
// `lib::HapticsController` without gaining access to its implementation
pub struct LofeltSdkController(HapticsController);

/// A collection of callbacks that the core uses to call back into native driver code
#[repr(C)]
pub struct Callbacks {
    /// Will be called for amplitude events streamed during pre-authored clip playback
    play_streaming_amplitude_event: extern "C" fn(*mut c_void, AmplitudeEvent),

    /// Will be called for frequency events streamed during pre-authored clip playback
    play_streaming_frequency_event: extern "C" fn(*mut c_void, FrequencyEvent),

    /// Will be called once when initializing the streaming thread, should increase the
    /// thread priority
    init_thread: extern "C" fn(),
}

static ONCE: Once = Once::new();

// Initialize the logger when called for the first time.
//
// This makes sure all calls to log::error!() end up written to stderr, prefixed
// with "LofeltHaptics" so that a developer can see that the log statement originated
// from our library.
//
// Note that we need to use the log crate, and not println!() or eprintln!(),
// as not all platforms capture stdout and stderr. This is the case for Android,
// where we use android_logger instead of env_logger to capture log::error!()
// calls - see jni_api.rs.
fn init_logging() {
    ONCE.call_once(|| {
        let mut builder = Builder::from_default_env();
        builder.format(|buf, record| {
            writeln!(
                buf,
                "[{}] LofeltHaptics::{} {}: {}",
                buf.timestamp_millis(),
                record.target(),
                record.level(),
                record.args()
            )
        });
        if let Err(err) = builder.try_init() {
            eprintln!("LofeltHaptics: Initializing the logger failed: {}", err);
        }
    });
}

/// Creates and returns a `LofeltSdkController`
///
/// Returns a null pointer on error, and `lofelt_sdk_get_error_message` can be called to get
/// additional information about the error.
///
/// # Arguments
/// * `native_driver` - a C void pointer to a native driver object that will be passed
///   to all callbacks.
/// * `callbacks` - the function pointers for the callbacks
#[no_mangle]
pub extern "C" fn lofelt_sdk_controller_create(
    native_driver: *mut c_void,
    callbacks: Callbacks,
) -> *mut LofeltSdkController {
    init_logging();

    let native_driver_for_callback = CVoidPtr(native_driver);
    let play_streaming_amplitude_event_for_callback = callbacks.play_streaming_amplitude_event;
    let play_streaming_amplitude_event = move |event: AmplitudeEvent| {
        play_streaming_amplitude_event_for_callback(native_driver_for_callback.0, event);
    };

    let native_driver_for_callback = CVoidPtr(native_driver);
    let play_streaming_frequency_event_for_callback = callbacks.play_streaming_frequency_event;
    let play_streaming_frequency_event = move |event: FrequencyEvent| {
        play_streaming_frequency_event_for_callback(native_driver_for_callback.0, event);
    };

    let init_thread_for_callback = callbacks.init_thread;
    let init_thread = move || {
        init_thread_for_callback();
    };

    let player = clip_players::streaming::Player::new(streaming::Callbacks {
        amplitude_event: Box::new(play_streaming_amplitude_event),
        frequency_event: Box::new(play_streaming_frequency_event),
        init_thread: Box::new(init_thread),
    });
    let player = match player {
        Ok(player) => player,
        Err(err) => {
            set_error(format!("Unable to create clip player: {}", err));
            return std::ptr::null_mut();
        }
    };

    let haptics_controller = HapticsController::new(Box::new(player));
    Box::into_raw(Box::new(LofeltSdkController(haptics_controller)))
}

/// Deallocates `LofeltSdkController` struct.
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_destroy(
    controller: *mut LofeltSdkController,
) -> c_int {
    if !controller.is_null() {
        Box::from_raw(controller);
        SUCCESS
    } else {
        set_error("Error destroying controller: \nController is null".to_string())
    }
}

/// Loads a haptic clip.
///
/// In addition to `ERROR`, `PARTIAL_VERSION_SUPPORT` can be returned.
///
/// The caller keeps ownership of `data` and is responsible for freeing the buffer.
///
/// # Arguments
/// * `data` - The JSON of the .haptic file, encoded as UTF-8, without a null terminator
/// * `data_size_bytes` - The amount of bytes in `data`
///
/// # Safety
/// - `data` needs to be a valid pointer to an array of bytes at least `data_size_bytes` bytes large
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_load(
    controller: &mut LofeltSdkController,
    data: *const c_char,
    data_size_bytes: usize,
) -> c_int {
    let data = slice::from_raw_parts(data as *const u8, data_size_bytes);
    let data = std::str::from_utf8(data);
    let data = match data {
        Ok(data) => data,
        Err(err) => return set_error(format!("Haptic data is not valid UTF-8: {}", err)),
    };

    match controller.0.load(data) {
        Ok(VersionSupport::Full) => SUCCESS,
        Ok(VersionSupport::Partial) => PARTIAL_VERSION_SUPPORT,
        Err(error) => set_error(format!("Error loading haptic data: \n{}", error)),
    }
}

/// Plays a haptic clip.
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_play(controller: &mut LofeltSdkController) -> c_int {
    match controller.0.play() {
        Ok(_) => SUCCESS,
        Err(error) => set_error(format!("Error playing haptic clip: \n{}", error)),
    }
}

/// Stops a previously played haptic clip.
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_stop(controller: &mut LofeltSdkController) -> c_int {
    match controller.0.stop() {
        Ok(_) => SUCCESS,
        Err(error) => set_error(format!("Error stopping haptic clip: \n{}", error)),
    }
}

/// Jumps to a position in the haptic clip.
///
/// # Arguments
/// * `time` - the new position within the clip, as seconds from the beginning of the clip
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_seek(
    controller: &mut LofeltSdkController,
    time: f32,
) -> c_int {
    match controller.0.seek(time) {
        Ok(_) => SUCCESS,
        Err(error) => set_error(format!(
            "Error seeking to position {:.3}s in haptic clip: \n{}",
            time, error
        )),
    }
}

/// Sets the amplitude multiplication for a haptic clip.
///
/// # Arguments
/// * `amplitude_multiplication` - the new multiplication factor
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_set_amplitude_multiplication(
    controller: &mut LofeltSdkController,
    amplitude_multiplication: f32,
) -> c_int {
    match controller
        .0
        .set_amplitude_multiplication(amplitude_multiplication)
    {
        Ok(_) => SUCCESS,
        Err(error) => set_error(format!(
            "Error setting amplitude multiplication to {:.2}: \n{}",
            amplitude_multiplication, error
        )),
    }
}

/// Sets the frequency shift for a haptic clip.
///
/// # Arguments
/// * `shift` - the new frequency shift
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_set_frequency_shift(
    controller: &mut LofeltSdkController,
    shift: f32,
) -> c_int {
    match controller.0.set_frequency_shift(shift) {
        Ok(_) => SUCCESS,
        Err(error) => set_error(format!(
            "Error setting frequency shift to {:.2}: \n{}",
            shift, error
        )),
    }
}

/// Sets the playback to repeat from the start when it reaches the end of a clip.
///
/// # Arguments
/// * `enabled` - Setting `enabled` to true enables looping; `false` sets the clip to be played
///   only once
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_loop(
    controller: &mut LofeltSdkController,
    enabled: bool,
) -> c_int {
    match controller.0.set_looping(enabled) {
        Ok(_) => SUCCESS,
        Err(error) => set_error(format!("Error enabling loop for haptic clip: \n{}", error)),
    }
}

/// Returns the duration of the loaded clip
///
/// It will return 0.0 in case the clip is not loaded
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_controller_get_clip_duration(
    controller: &mut LofeltSdkController,
) -> c_float {
    controller.0.get_clip_duration()
}

/// Returns the length of the last error message in bytes, or 0 if there is no last
/// error message.
///
/// `lofelt_sdk_get_error_message` expects to receive a pre-allocated buffer of adequate size for
/// the error message; this function provides the expected size.
///
/// The calculated length includes a null-terminator.
#[no_mangle]
pub extern "C" fn lofelt_sdk_get_error_message_length() -> c_int {
    get_error_message_length()
}

/// Writes the error message to the buffer that the client passes in.
///
/// An error will cause ERROR to be returned.
///
/// # Safety
/// The client can ensure that the `buffer` is large enough by calling get_error_message_length().
/// The string data returned will be null-terminated and in UTF-8 format.
#[no_mangle]
pub unsafe extern "C" fn lofelt_sdk_get_error_message(buffer: *mut c_char, length: c_int) -> c_int {
    get_error_message(buffer, length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[no_mangle]
    pub extern "C" fn play_streaming_amplitude_event_dummy(
        _: *mut std::ffi::c_void,
        _: AmplitudeEvent,
    ) {
    }

    #[no_mangle]
    pub extern "C" fn play_streaming_frequency_event_dummy(
        _: *mut std::ffi::c_void,
        _: FrequencyEvent,
    ) {
    }

    #[no_mangle]
    pub extern "C" fn init_thread_dummy() {}

    #[test]
    fn check_errors_play() {
        let callbacks = Callbacks {
            play_streaming_amplitude_event: play_streaming_amplitude_event_dummy,
            play_streaming_frequency_event: play_streaming_frequency_event_dummy,
            init_thread: init_thread_dummy,
        };
        let controller = lofelt_sdk_controller_create(std::ptr::null_mut(), callbacks);
        unsafe {
            if lofelt_sdk_controller_play(&mut *controller) == SUCCESS {
                panic!("Should return an Error");
            } else if lofelt_sdk_get_error_message_length() <= 0 {
                panic!("Error message length should be > 0");
            } // TODO: Test getting error string with lofelt_sdk_get_error_message
        }
    }

    #[test]
    fn check_errors_load() {
        let callbacks = Callbacks {
            play_streaming_amplitude_event: play_streaming_amplitude_event_dummy,
            play_streaming_frequency_event: play_streaming_frequency_event_dummy,
            init_thread: init_thread_dummy,
        };
        let controller = lofelt_sdk_controller_create(std::ptr::null_mut(), callbacks);
        unsafe {
            let data: [c_char; 1] = [0; 1];
            if lofelt_sdk_controller_load(&mut *controller, &data as *const i8, 1) == SUCCESS {
                panic!("Should return an Error");
            } else if lofelt_sdk_get_error_message_length() <= 0 {
                panic!("Error message length should be > 0");
            }
        }
    }
}
