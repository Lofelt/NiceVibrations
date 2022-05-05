// See README.md for a high-level documentation of this crate.

mod algorithm;
pub mod api;

// Contains a vibration pattern to make a gamepad rumble.
//
// This is the Rust equivalent to GamepadRumble in Gamepad.cs, see the documentation there
// for more details.
#[derive(Debug, PartialEq, Clone)]
pub struct GamepadRumble {
    durations_ms: Vec<i32>,
    low_frequency_motor_speeds: Vec<f32>,
    high_frequency_motor_speeds: Vec<f32>,
}
