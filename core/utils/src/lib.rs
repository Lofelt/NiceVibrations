// Copyright (c) Meta Platforms, Inc. and affiliates.

use std::fmt::Display;

pub mod test_utils;

#[derive(Debug, PartialEq)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error {
            message: message.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[cfg(target_os = "android")]
impl From<jni::errors::Error> for Error {
    fn from(error: jni::errors::Error) -> Self {
        Error::new(&error.to_string())
    }
}

/// Returns a value that is linearly interpolated between value_a and value_b.
///
/// The weight is calculated from the position of `time` in the interval `[time_a, time_b]`.
pub fn interpolate(time_a: f32, time_b: f32, value_a: f32, value_b: f32, time: f32) -> f32 {
    debug_assert!(time_b >= time_a, "time_b needs to be after time_a");
    debug_assert!(
        time >= time_a && time <= time_b,
        "The time value needs to be within the interval [time_a, time_b]"
    );
    let time_diff = time_b - time_a;
    if time_diff == 0.0 {
        return value_b;
    }
    let value_diff = value_b - value_a;
    let factor = (time - time_a) / time_diff;
    value_a + value_diff * factor
}

#[cfg(test)]
mod tests {
    #[test]
    // Test linear interpolation
    fn interpolate() {
        assert!((super::interpolate(0.5, 1.0, 2.0, 5.0, 0.5) - 2.0) <= f32::EPSILON);
        assert!((super::interpolate(0.5, 1.0, 2.0, 5.0, 0.75) - 3.5) <= f32::EPSILON);
        assert!((super::interpolate(0.5, 1.0, 2.0, 5.0, 1.0) - 5.0) <= f32::EPSILON);
    }
}
