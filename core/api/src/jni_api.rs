// Copyright (c) Meta Platforms, Inc. and affiliates.

#![cfg(target_os = "android")]

use android_logger::{Config, FilterBuilder};
use core::f32;
use jni::{
    objects::{JObject, JValue},
    sys::jfloat,
    sys::{jboolean, jbyteArray, jlong},
    JNIEnv,
};
use lib::{
    clip_players::android::{Callbacks, Player},
    HapticsController,
};
use log::{Level, LevelFilter};
use std::{os::raw::c_char, slice, sync::Once};

static ONCE: Once = Once::new();

pub struct LofeltSdkController(HapticsController);

/// # `xyz_inner()` methods
///
/// This file contains some `xyz_inner()` methods. The purpose of these is to simplify
/// error handling. Within the `xyz_inner()` methods, the questionmark operator can be used,
/// as the `xyz_inner()` methods all return `Result<_, lib::Error>`. Using the questionmark
/// operator is not possible in the outer methods, as the JNI method signatures can not contain
/// `Result`.
///
/// # Controller handle
///
/// Every function uses a `HapticsController`. It is created by `Java_com_lofelt_haptics_LofeltHaptics_create()`,
/// and then returned to the Java layer as a pointer casted to a `jlong`. The Java layer
/// keeps hold of that handle to `HapticsController` and passes it to every function as a
/// parameter. The functions then cast the `jlong` back to a `HapticsController` and uses it.
///
/// # Error handling
///
/// ## Errors when Java called into Rust
///
/// When an error occurs, the inner functions return it as a `Result<_, lib::Error>` to the
/// outer functions. The outer functions then raise a `java.lang.RuntimeException` that is
/// thrown as soon as the Rust function returns back to Java.
///
/// ## Errors when Rust called into Java, for the callbacks
///
/// The Java callbacks called from Rust can raise exceptions. After each call into Java,
/// we check if there is an exception, clear the exception and convert it to a
/// `Result<(), lib::Error>`. That `Result` is then returned to the caller. Because the caller
/// is the haptic thread, it has nowhere to further return it to, and logs the error.

// Throws an exception created from `error`
//
// However, throw_exception() doesn't actually throw an exception in Rust, as Rust doesn't have exceptions.
// This sets a flag to tell Java that an exception should be raised but JNI Rust functions with calls to
// throw_exception() still need to return the appropriate declared JNI return type.
// The Java exception is thrown by the JVM as soon as all the Rust layer returns to the Java layer.
// In other words it means something like "Hey Java, please throw an exception once we've returned
// from the Rust land, please".
fn throw_exception(env: &JNIEnv, error: lib::Error) {
    // If there is already an active exception, don't throw a new one, as invoking
    // JNI functions is not allowed while an exception is active
    if let Ok(exception_occurred) = env.exception_check() {
        if exception_occurred {
            log::error!("Unable to throw an exception, as there already is an active exception.");
            return;
        }
    }

    let throw_result = env.throw_new("java/lang/RuntimeException", &error.message);
    if throw_result.is_err() {
        log::error!(
            "Throwing exception failed: {}. Error: {}",
            throw_result.unwrap_err().to_string(),
            error
        );
    } else {
        log::error!("Error in core, thrown to Java: {}", error);
    }
}

// Converts a java/lang/String object into a Rust String
fn java_string_to_rust(env: jni::AttachGuard, string: JValue) -> Result<String, lib::Error> {
    let string = string.l()?;
    let string = env.get_string(string.into())?;
    let string = string
        .to_str()
        .map_err(|e| lib::Error::new(&format!("UTF-8 conversion error: {}", e)))?;
    Ok(string.to_owned())
}

// Checks if a Java exception has occurred and returns it as `Err` if that's the case
//
// This is intended to be used after a call into Java. `call_result` is the result
// of the Java call, and is returned back if the call had an error.
fn handle_exception_from_call(
    env: jni::AttachGuard,
    call_result: jni::errors::Result<JValue>,
) -> Result<(), lib::Error> {
    let throwable = env.exception_occurred()?;
    if throwable.is_null() {
        // No exception occurred, just convert and return `call_result`
        call_result?;
        return Ok(());
    }

    // The first thing we need to do, before calling into any other JNI function, is to clear
    // the exception. Otherwise Android would terminate the process.
    env.exception_clear()?;

    // Get the message text of the exception by calling Throwable::getMessage(). This text
    // is then included in the Result's error message, so that we have more detail than just
    // "An exception occurred".
    let get_message_result = env.call_method(throwable, "getMessage", "()Ljava/lang/String;", &[]);

    // Trying to call getMessage() might have thrown an exception itself. Just ignore that exception
    // here.
    if env.exception_check()? {
        env.exception_clear()?
    }

    let exception_message = java_string_to_rust(env, get_message_result?)?;
    Err(lib::Error::new(&format!(
        "An exception occurred: {}",
        exception_message
    )))
}

fn get_controller<'a>(controller_handle: jlong) -> Result<&'a mut HapticsController, lib::Error> {
    if controller_handle == 0 {
        return Err(lib::Error::new("Controller is null"));
    }
    let controller = unsafe { &mut *(controller_handle as *mut HapticsController) };
    Ok(controller)
}

fn create_inner(env: &JNIEnv, callback_object: JObject) -> Result<jlong, lib::Error> {
    let load_callback = {
        // The callback captures a JavaVM from which it can get a JNIEnv again.
        // This is needed because the callback can not capture the JNIEnv directly,
        // as it has a lifetime.
        let jvm = env.get_java_vm()?;

        // The callback captures a reference to callback_object. Because callback_object
        // has a lifetime and is only valid until the Rust code returns back to Java, a
        // global reference is created and captured instead.
        let callback_object_global_ref = env.new_global_ref(callback_object)?;

        // Create a closure that matches the signature needed by `Callbacks::new()`.
        // `jvm` and `callback_object_global_ref` are captured and therefore not part
        // of the signature.
        move |timings: &[i64], amplitudes: &[i32], enabled: bool| -> Result<(), lib::Error> {
            // Get back the JNIEnv (or rather, a wrapper around it) from the JavaVM
            let env = jvm.attach_current_thread()?;

            // Convert the timings and amplitudes arrays to Java arrays.
            // See also https://www3.ntu.edu.sg/home/ehchua/programming/java/JavaNativeInterface.html#zz-4.3.
            let timings_java = env.new_long_array(timings.len() as i32)?;
            let amplitudes_java = env.new_int_array(amplitudes.len() as i32)?;
            env.set_long_array_region(timings_java, 0, timings)?;
            env.set_int_array_region(amplitudes_java, 0, amplitudes)?;

            // Call the Java method.
            // See also http://journals.ecs.soton.ac.uk/java/tutorial/native1.1/implementing/method.html.
            let result = env.call_method(
                &callback_object_global_ref,
                "loadCallback",
                "([J[IZ)V",
                &[timings_java.into(), amplitudes_java.into(), enabled.into()],
            );
            handle_exception_from_call(env, result)
        }
    };
    let play_callback = {
        let jvm = env.get_java_vm()?;
        let callback_object_global_ref = env.new_global_ref(callback_object)?;
        move || -> Result<(), lib::Error> {
            let env = jvm.attach_current_thread()?;
            let result = env.call_method(&callback_object_global_ref, "playCallback", "()V", &[]);
            handle_exception_from_call(env, result)
        }
    };
    let stop_callback = {
        let jvm = env.get_java_vm()?;
        let callback_object_global_ref = env.new_global_ref(callback_object)?;
        move || -> Result<(), lib::Error> {
            let env = jvm.attach_current_thread()?;
            let result = env.call_method(&callback_object_global_ref, "stopCallback", "()V", &[]);
            handle_exception_from_call(env, result)
        }
    };
    let unload_callback = {
        let jvm = env.get_java_vm()?;
        let callback_object_global_ref = env.new_global_ref(callback_object)?;
        move || -> Result<(), lib::Error> {
            let env = jvm.attach_current_thread()?;
            let result = env.call_method(&callback_object_global_ref, "unloadCallback", "()V", &[]);
            handle_exception_from_call(env, result)
        }
    };

    let seek_callback = {
        let jvm = env.get_java_vm()?;
        let callback_object_global_ref = env.new_global_ref(callback_object)?;

        // Create a closure that matches the signature needed by `Callbacks::new()`.
        // `jvm` and `callback_object_global_ref` are captured and therefore not part
        // of the signature.
        move |timings: &[i64], amplitudes: &[i32]| -> Result<(), lib::Error> {
            // Get back the JNIEnv (or rather, a wrapper around it) from the JavaVM
            let env = jvm.attach_current_thread()?;

            // Convert the timings and amplitudes arrays to Java arrays.
            // See also https://www3.ntu.edu.sg/home/ehchua/programming/java/JavaNativeInterface.html#zz-4.3.
            let timings_java = env.new_long_array(timings.len() as i32)?;
            let amplitudes_java = env.new_int_array(amplitudes.len() as i32)?;
            env.set_long_array_region(timings_java, 0, timings)?;
            env.set_int_array_region(amplitudes_java, 0, amplitudes)?;

            // Call the Java method.
            // See also http://journals.ecs.soton.ac.uk/java/tutorial/native1.1/implementing/method.html.
            let result = env.call_method(
                &callback_object_global_ref,
                "seekCallback",
                "([J[I)V",
                &[timings_java.into(), amplitudes_java.into()],
            );
            handle_exception_from_call(env, result)
        }
    };

    let player = Player::new(Callbacks::new(
        load_callback,
        play_callback,
        stop_callback,
        unload_callback,
        seek_callback,
    ))?;
    let controller = HapticsController::new(Box::new(player));
    let raw_controller_handle = Box::into_raw(Box::new(controller));
    Ok(raw_controller_handle as jlong)
}

/// Creates a `HapticsController` and returns an opaque handle to it.
///
/// Logging is also initialized in the first call to this.
///
/// The load, play, stop and unload callbacks will be invoked on `callback_object`. For this,
/// a global reference to that object is kept, which means that the object will not be
/// garbage-collected until the global reference is released in
/// `Java_com_lofelt_haptics_LofeltHaptics_destroy()`.
#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_create(
    env: JNIEnv,
    _caller: JObject,
    callback_object: JObject,
) -> jlong {
    ONCE.call_once(|| {
        android_logger::init_once(
            Config::default()
                // Disable JNI-internal logs, which are quite noisy
                .with_filter(
                    FilterBuilder::new()
                        .filter_level(LevelFilter::Trace)
                        .filter_module("jni", LevelFilter::Warn)
                        .filter_module(module_path!(), LevelFilter::Trace)
                        .build(),
                )
                .with_min_level(Level::Trace)
                .with_tag("lofelt-sdk-core"),
        );
        log_panics::init();
    });

    let result = create_inner(&env, callback_object);
    match result {
        Ok(controller_handle) => controller_handle,
        Err(err) => {
            throw_exception(&env, err);
            -1
        }
    }
}

fn destroy_inner(controller_handle: jlong) -> Result<(), lib::Error> {
    let controller = get_controller(controller_handle)?;
    unsafe { let _ = Box::from_raw(controller); };
    Ok(())
}

/// Destroys the `HapticsController` represented by `controller_handle`.
///
/// This also releases the reference to the callback object passed to
/// `Java_com_lofelt_haptics_LofeltHaptics_create()`.
#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_destroy(
    env: JNIEnv,
    _caller: JObject,
    controller_handle: jlong,
) {
    let result = destroy_inner(controller_handle);
    if let Err(err) = result {
        throw_exception(&env, err);
    }
}

// # Safety
// See `lofeltHapticsLoadDirect()`
pub fn load_direct_inner(
    controller: *mut LofeltSdkController,
    clip: *const c_char,
    clip_size_bytes: usize,
) -> Result<(), lib::Error> {
    let controller = &mut unsafe { controller.as_mut() }
        .ok_or_else(|| lib::Error::new("Invalid controller handle"))?
        .0;
    let clip = unsafe { slice::from_raw_parts(clip as *const u8, clip_size_bytes) };
    let clip = std::str::from_utf8(clip)
        .map_err(|err| lib::Error::new(&format!("Clip is not valid UTF-8: {}", err)))?;
    controller.load(clip)?;
    Ok(())
}

// C API for directly loading a clip, bypassing the JNI API
// `Java_com_lofelt_haptics_LofeltHaptics_load()`.
//
// This is called from the Unity asset scripts when loading a clip. For all other operations
// apart from loading a clip, Unity calls into the Java API, which in turn calls the
// JNI API in this file.
//
// The reason Unity calls this C API directly, bypassing Java, is performance. If Unity
// were to use the Java API for loading a clip, the clip needs to be converted from
// C#'s byte[] to Java's byte[], which is really slow. By calling the C API directly,
// no conversion happens, bringing a major speed improvement.
//
// The JNI and Java APIs for loading a clip still exist in parallel to the C API here,
// for users of the Android SDK.
//
// The passed `clip` needs to be valid UTF-8 without a null terminator.
//
// The caller keeps ownership of `clip` and is responsible for freeing the buffer.
//
// # Error handling
// Any error is just logged, and not returned to the caller. This matches our error
// handling strategy in Unity, which is logging errors instead of throwing exceptions.
//
// # Safety
// - `clip` needs to be a valid pointer to an array of bytes at least `clip_size_bytes` bytes large
// - `controller` needs to be a valid pointer to a `HapticsController`
#[no_mangle]
pub extern "system" fn lofeltHapticsLoadDirect(
    controller: *mut LofeltSdkController,
    clip: *const c_char,
    clip_size_bytes: usize,
) {
    if let Err(err) = load_direct_inner(controller, clip, clip_size_bytes) {
        log::error!("Failed to load clip: {}", err);
    }
}

fn load_inner(env: &JNIEnv, controller_handle: jlong, clip: jbyteArray) -> Result<(), lib::Error> {
    let controller = get_controller(controller_handle)?;
    let clip = env.convert_byte_array(clip)?;
    let clip = std::str::from_utf8(&clip)
        .map_err(|err| lib::Error::new(&format!("Reading clip data as UTF-8 failed: {}", err)))?;
    controller.load(clip)?;
    Ok(())
}

/// Loads a .haptic clip from its UTF-8 encoded JSON string.
///
/// `clip` must not contain a null terminator.
///
/// The caller keeps ownership of `clip` and is responsible for freeing the buffer.
#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_load(
    env: JNIEnv,
    _caller: JObject,
    controller_handle: jlong,
    clip: jbyteArray,
) {
    let result = load_inner(&env, controller_handle, clip);
    if let Err(err) = result {
        throw_exception(&env, err);
    }
}

fn play_inner(controller_handle: jlong) -> Result<(), lib::Error> {
    let controller = get_controller(controller_handle)?;
    controller.play()
}

/// Plays a haptic clip previously loaded with `Java_com_lofelt_haptics_LofeltHaptics_load()`.
#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_play(
    env: JNIEnv,
    _caller: JObject,
    controller_handle: jlong,
) {
    let result = play_inner(controller_handle);
    if let Err(err) = result {
        throw_exception(&env, err);
    }
}

fn stop_inner(controller_handle: jlong) -> Result<(), lib::Error> {
    let controller = get_controller(controller_handle)?;
    controller.stop()
}

/// Stops a haptic clip previously played with `Java_com_lofelt_haptics_LofeltHaptics_play()`.
#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_stop(
    env: JNIEnv,
    _caller: JObject,
    controller_handle: jlong,
) {
    let result = stop_inner(controller_handle);
    if let Err(err) = result {
        throw_exception(&env, err);
    }
}

fn seek_inner(controller_handle: jlong, seek_time: jfloat) -> Result<(), lib::Error> {
    let controller = get_controller(controller_handle)?;
    controller.seek(seek_time)
}

/// Seeks to a position in the clip
///
/// Negative times are clamped to zero.
#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_seek(
    env: JNIEnv,
    _caller: JObject,
    controller_handle: jlong,
    seek_time: jfloat,
) {
    let result = seek_inner(controller_handle, seek_time);
    if let Err(err) = result {
        throw_exception(&env, err);
    }
}

fn set_amplitude_multiplication_inner(
    controller_handle: jlong,
    amplitude_multiplication: jfloat,
) -> Result<(), lib::Error> {
    let controller = get_controller(controller_handle)?;
    controller.set_amplitude_multiplication(amplitude_multiplication)
}

#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_setAmplitudeMultiplication(
    env: JNIEnv,
    _caller: JObject,
    controller_handle: jlong,
    amplitude_multiplication: jfloat,
) {
    let result = set_amplitude_multiplication_inner(controller_handle, amplitude_multiplication);
    if let Err(err) = result {
        throw_exception(&env, err);
    }
}

fn loop_inner(controller_handle: jlong, enabled: jboolean) -> Result<(), lib::Error> {
    let controller = get_controller(controller_handle)?;
    controller.set_looping(enabled != 0)
}

/// Sets the playback to repeat from the start at the end of the clip
#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_loop(
    env: JNIEnv,
    _caller: JObject,
    controller_handle: jlong,
    enabled: jboolean,
) {
    let result = loop_inner(controller_handle, enabled);
    if let Err(err) = result {
        throw_exception(&env, err);
    }
}

fn get_clip_duration(controller_handle: jlong) -> Result<f32, lib::Error> {
    let controller = get_controller(controller_handle)?;
    Ok(controller.get_clip_duration())
}

/// Returns the duration of a loaded clip
#[no_mangle]
pub extern "system" fn Java_com_lofelt_haptics_LofeltHaptics_getClipDuration(
    env: JNIEnv,
    _caller: JObject,
    controller_handle: jlong,
) -> jfloat {
    let result = get_clip_duration(controller_handle);
    match result {
        Ok(duration) => duration as jfloat,
        Err(err) => {
            throw_exception(&env, err);
            0.0_f32
        }
    }
}
