use crate::GamepadRumble;
use datamodel::{
    emphasis::emphasize,
    interpolation::{InterpolationParameters, Interpolator},
    v1::{AmplitudeBreakpoint, FrequencyBreakpoint},
    waveform::{Waveform, WaveformConversionParameters},
};
use std::cmp::Ordering;
use utils::Error;

// Takes a list of frequency breakpoints and converts them to a list of amplitude breakpoints
fn frequency_breakpoints_to_amplitude_breakpoints(
    frequency_breakpoints: &Option<Vec<FrequencyBreakpoint>>,
) -> Vec<AmplitudeBreakpoint> {
    match frequency_breakpoints {
        Some(frequency_breakpoints) => frequency_breakpoints
            .iter()
            .map(|frequency_breakpoint| AmplitudeBreakpoint {
                time: frequency_breakpoint.time,
                amplitude: frequency_breakpoint.frequency,
                emphasis: None,
            })
            .collect(),
        None => Vec::new(),
    }
}

// Extends the given list of breakpoints by adding new breakpoints at every
// time point of `extended_timings`.
//
// The amplitude of a new breakpoint is simply an interpolation of the breakpoint
// before and after the new time point. That makes the new breakpoints redundant,
// no new information is added.
//
// The algorithmic complexity here is inefficient, with a more clever algorithm the
// calls to position() could probably be eliminated. Since this is only called when
// importing clips in the Unity editor, the performance isn't critical though.
fn extend_breakpoints(
    breakpoints: &[AmplitudeBreakpoint],
    extended_timings: &[f32],
) -> Vec<AmplitudeBreakpoint> {
    let mut new_breakpoints = Vec::new();

    // Iterate over each time point and create a new breakpoint for that time point
    for &time in extended_timings {
        // If a breakpoint already exists at this time point, do nothing
        let breakpoint_at_time_exists = breakpoints
            .iter()
            .any(|breakpoint| (breakpoint.time - time).abs() <= f32::EPSILON);
        if breakpoint_at_time_exists {
            continue;
        }

        // No breakpoint at this time point exists yet. Find the breakpoints before and after
        // the time point, and then create a new breakpoint based on the interpolation of the
        // two (before and after) breakpoints.
        let breakpoint_after_index = breakpoints
            .iter()
            .position(|breakpoint| breakpoint.time >= time);
        let new_breakpoint = match breakpoint_after_index {
            Some(breakpoint_after_index) => {
                if breakpoint_after_index == 0 {
                    // The time point is before all existing breakpoints. In that case, create
                    // a breakpoint with amplitude 0.
                    AmplitudeBreakpoint {
                        time,
                        amplitude: 0.0,
                        emphasis: None,
                    }
                } else {
                    let breakpoint_before_index = breakpoint_after_index - 1;
                    let breakpoint_before = &breakpoints[breakpoint_before_index];
                    let breakpoint_after = &breakpoints[breakpoint_after_index];
                    AmplitudeBreakpoint::from_interpolated_breakpoints(
                        breakpoint_before,
                        breakpoint_after,
                        time,
                    )
                }
            }
            None => {
                // The time point is after all existing breakpoints. In that case, create a
                // breakpoint with the same amplitude as the last one.
                AmplitudeBreakpoint {
                    time,
                    amplitude: breakpoints
                        .last()
                        .map_or(0.0, |breakpoint| breakpoint.amplitude),
                    emphasis: None,
                }
            }
        };

        new_breakpoints.push(new_breakpoint);
    }

    // Add the old and new breakpoints together, sort, and return
    let mut result = Vec::with_capacity(breakpoints.len() + new_breakpoints.len());
    result.append(&mut breakpoints.to_owned());
    result.append(&mut new_breakpoints);
    result.sort_by(|breakpoint_a, breakpoint_b| {
        debug_assert!(breakpoint_a.time.partial_cmp(&breakpoint_b.time).is_some());
        breakpoint_a
            .time
            .partial_cmp(&breakpoint_b.time)
            .unwrap_or(Ordering::Equal)
    });
    result
}

// Extracts the times of the given breakpoints into a list and returns that list
fn breakpoint_times(breakpoints: &[AmplitudeBreakpoint]) -> Vec<f32> {
    breakpoints
        .iter()
        .map(|breakpoint| breakpoint.time)
        .collect()
}

// Convert a haptic clip given as a JSON string to a GamepadRumble.
//
// The Gamepad API in Unity models the gamepad as having two motors: A low frequency motor and a
// high frequency motor. The low frequency motor is usually on the left, and the high frequency
// motor on the right.
//
// The algorithm we use to convert from .haptic to a GamepadRumble is a bit silly and doesn't make
// much sense:
// - The amplitude envelope is used as the motor speeds of the low frequency motor. The higher
//   the amplitude of a breakpoint, the higher the motor speed.
// - The frequency envelope is used as the motor speeds of the high frequency motor. The higher
//   the frequency of a breakpoint, the higher the motor speed.
// We use this algorithm only because we didn't want to invest time in coming up with a better
// algorithm, and because it's the algorithm used by Nice Vibrations 3.9.
pub fn convert_haptic_to_gamepad_rumble_inner(data: &[u8]) -> Result<GamepadRumble, Error> {
    // Step 1: Convert bytes to DataModel
    let data = std::str::from_utf8(data).map_err(|err| {
        Error::new(&format!(
            "Failed to convert haptic clip data to UTF-8: {}",
            err
        ))
    })?;
    let (_, data) = datamodel::latest_from_json(data)
        .map_err(|err| Error::new(&format!("Failed to load haptic clip: {}", err)))?;

    // Step 2: Convert frequency envelope to list of amplitude breakpoints
    let low_frequency_motor_breakpoints = data.signals.continuous.envelopes.amplitude;
    let high_frequency_motor_breakpoints = frequency_breakpoints_to_amplitude_breakpoints(
        &data.signals.continuous.envelopes.frequency,
    );

    // Step 3: Add emphasis to breakpoints
    let low_frequency_motor_breakpoints =
        emphasize(&low_frequency_motor_breakpoints, Default::default());
    let high_frequency_motor_breakpoints =
        emphasize(&high_frequency_motor_breakpoints, Default::default());

    // Step 4: Interpolate breakpoints.
    // This is needed because our GamepadRumbler in Unity does not interpolate on its own

    // Use the same value like on Android for Q_BITS, but use a MIN_TIME_STEP of ~16ms. 16ms is the
    // minimum resolution that GamepadRumbler can play on platforms with a refresh rate of 60Hz.
    const Q_BITS: u32 = 8;
    const MAX_WAVEFORM_AMPLITUDE: i32 = 2_i32.pow(Q_BITS) - 1;
    const MIN_TIME_STEP: f32 = 1.0 / 60.0;

    let interpolator = Interpolator::new(InterpolationParameters::new(Q_BITS, MIN_TIME_STEP));
    let low_frequency_motor_breakpoints = interpolator.process(&low_frequency_motor_breakpoints);
    let high_frequency_motor_breakpoints = interpolator.process(&high_frequency_motor_breakpoints);

    // Step 5: Ensure low and high frequency breakpoint vectors have the same amount of breakpoints.
    // This is needed because both vectors in GamepadRumble need to have the same length in order
    // for it to be a valid GamepadRumble. In GamepadRumble, there is only one durations vector that
    // represents the durations of both motor speed vectors, so all vector lengths need to match up.
    let low_frequency_motor_breakpoints = extend_breakpoints(
        &low_frequency_motor_breakpoints,
        &breakpoint_times(&high_frequency_motor_breakpoints),
    );
    let high_frequency_motor_breakpoints = extend_breakpoints(
        &high_frequency_motor_breakpoints,
        &breakpoint_times(&low_frequency_motor_breakpoints),
    );

    // TODO: Add a step to reduce the amount of breakpoints

    // Step 6: Convert breakpoints to a Waveform
    let low_frequency_motor_waveform = Waveform::from_breakpoints(
        &low_frequency_motor_breakpoints,
        WaveformConversionParameters {
            max_amplitude: MAX_WAVEFORM_AMPLITUDE,
        },
    );
    let high_frequency_motor_waveform = Waveform::from_breakpoints(
        &high_frequency_motor_breakpoints,
        WaveformConversionParameters {
            max_amplitude: MAX_WAVEFORM_AMPLITUDE,
        },
    );

    // Step 7: Convert Waveform to GamepadRumble
    let gamepad_rumble = waveforms_to_gamepad_rumble(
        low_frequency_motor_waveform,
        high_frequency_motor_waveform,
        MAX_WAVEFORM_AMPLITUDE,
    )?;

    Ok(gamepad_rumble)
}

// Converts two waveforms to a GamepadRumble.
//
// The amplitudes of the waveforms are used as the motor speeds for the GamepadRumble.
// The main difference is that the amplitude of a Waveform goes from 0 to max_waveform_amplitude,
// while the motor speed of a GamepadRumble goes from 0.0 to 1.0.
fn waveforms_to_gamepad_rumble(
    low_frequency_motor_waveform: Waveform,
    high_frequency_motor_waveform: Waveform,
    max_waveform_amplitude: i32,
) -> Result<GamepadRumble, Error> {
    let entry_count = low_frequency_motor_waveform.timings.len();
    if high_frequency_motor_waveform.timings.len() != entry_count
        || low_frequency_motor_waveform.amplitudes.len() != entry_count
        || high_frequency_motor_waveform.amplitudes.len() != entry_count
    {
        return Err(Error::new("Internal error, waveform lengths don't match"));
    }

    let mut gamepad_rumble = GamepadRumble {
        durations_ms: Vec::with_capacity(entry_count),
        low_frequency_motor_speeds: Vec::with_capacity(entry_count),
        high_frequency_motor_speeds: Vec::with_capacity(entry_count),
    };

    for i in 0..entry_count {
        let timing = low_frequency_motor_waveform.timings[i];
        let low_freq_amplitude = low_frequency_motor_waveform.amplitudes[i];
        let high_freq_amplitude = high_frequency_motor_waveform.amplitudes[i];
        let low_freq_motor_speed = low_freq_amplitude as f32 / max_waveform_amplitude as f32;
        let high_freq_motor_speed = high_freq_amplitude as f32 / max_waveform_amplitude as f32;

        gamepad_rumble.durations_ms.push(timing as i32);
        gamepad_rumble
            .low_frequency_motor_speeds
            .push(low_freq_motor_speed);
        gamepad_rumble
            .high_frequency_motor_speeds
            .push(high_freq_motor_speed);
    }
    Ok(gamepad_rumble)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datamodel::test_utils::{amp, rounded_amplitude_breakpoints};
    use std::path::Path;
    use utils::test_utils::rounded_f32;

    // Loads a haptic clip from src/test_data/ and converts it to GamepadRumble
    fn load_from_test_data(path: &str) -> GamepadRumble {
        let haptic_clip_data = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/test_data")
                .join(path),
        )
        .unwrap();
        convert_haptic_to_gamepad_rumble_inner(&haptic_clip_data).unwrap()
    }

    // Rounds the floating point values in a GamepadRumble, so that two GamepadRumble objects can
    // be compared without running into rounding problems.
    fn rounded_gamepad_rumble(gamepad_rumble: &GamepadRumble) -> GamepadRumble {
        let mut result = gamepad_rumble.clone();
        for low_frequency_motor_speed in &mut result.low_frequency_motor_speeds {
            *low_frequency_motor_speed = rounded_f32(*low_frequency_motor_speed, 3);
        }
        for high_frequency_motor_speed in &mut result.high_frequency_motor_speeds {
            *high_frequency_motor_speed = rounded_f32(*high_frequency_motor_speed, 3);
        }
        result
    }

    // Prints the GamepadRumble as CSV format, so that the output can be copy & pasted
    // into a spreadsheet.
    // In the spreadsheet a graph can be used to visualize the GamepadRumble. This is useful
    // for debugging.
    #[allow(dead_code)]
    fn print_to_csv(gamepad_rumble: &GamepadRumble) {
        println!("time[ms], low freq speed, high freq speed");
        let mut time_point_ms = 0;
        for i in 0..gamepad_rumble.durations_ms.len() {
            println!(
                "{},{},{}",
                time_point_ms,
                gamepad_rumble.low_frequency_motor_speeds[i],
                gamepad_rumble.high_frequency_motor_speeds[i]
            );
            time_point_ms += gamepad_rumble.durations_ms[i];
            println!(
                "{},{},{}",
                time_point_ms,
                gamepad_rumble.low_frequency_motor_speeds[i],
                gamepad_rumble.high_frequency_motor_speeds[i]
            );
        }
    }

    #[test]
    fn amplitude_only() {
        let gamepad_rumble = load_from_test_data("amplitude_only.haptic");
        let expected_gamepad_rumble = GamepadRumble {
            durations_ms: vec![25, 25, 25, 25, 50],
            low_frequency_motor_speeds: vec![0.0, 0.24705882, 0.49803922, 0.7490196, 1.0],
            high_frequency_motor_speeds: vec![0.0, 0.0, 0.0, 0.0, 0.0],
        };
        assert_eq!(
            rounded_gamepad_rumble(&gamepad_rumble),
            rounded_gamepad_rumble(&expected_gamepad_rumble)
        );
    }

    #[test]
    #[allow(clippy::approx_constant)] // No clippy, 0.318 is not π
    fn emphasis() {
        let gamepad_rumble = load_from_test_data("emphasis.haptic");
        let expected_gamepad_rumble = GamepadRumble {
            durations_ms: vec![20, 30, 30, 30, 20, 20],
            low_frequency_motor_speeds: vec![0.2, 0.0, 1.0, 0.0, 0.239, 0.318],
            high_frequency_motor_speeds: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        };
        assert_eq!(
            rounded_gamepad_rumble(&gamepad_rumble),
            rounded_gamepad_rumble(&expected_gamepad_rumble)
        );
    }

    // Tests a clip in which the frequency breakpoints are at the same time points as the
    // amplitude breakpoints.
    // This tests the case in which the call to extend_breakpoints() in our algorithm is redundant.
    #[test]
    fn same_amplitude_frequency_timings() {
        let gamepad_rumble = load_from_test_data("same_amplitude_frequency_timings.haptic");
        let expected_gamepad_rumble = GamepadRumble {
            durations_ms: vec![25, 25, 25, 25, 50],
            low_frequency_motor_speeds: vec![0.0, 0.24705882, 0.49803922, 0.7490196, 1.0],
            high_frequency_motor_speeds: vec![1.0, 0.7490196, 0.49803922, 0.24705882, 0.0],
        };
        assert_eq!(
            rounded_gamepad_rumble(&gamepad_rumble),
            rounded_gamepad_rumble(&expected_gamepad_rumble)
        );
    }

    // Tests a clip in which the frequency breakpoints are at different time points as the
    // amplitude breakpoints.
    // This tests the case in which the call to extend_breakpoints() in our algorithm produces new
    // breakpoints.
    // TODO: This test produces a GamepadRumble with many entries close to each other,
    //                 which could be improved.
    #[test]
    fn different_amplitude_frequency_timings() {
        let gamepad_rumble = load_from_test_data("different_amplitude_frequency_timings.haptic");
        let expected_gamepad_rumble = GamepadRumble {
            durations_ms: vec![5, 5, 7, 10, 6, 12, 5, 12, 5, 13, 3, 14, 3, 15, 2, 16, 17],
            low_frequency_motor_speeds: vec![
                0.0, 0.03137255, 0.0627451, 0.10980392, 0.18039216, 0.21960784, 0.29803923,
                0.33333334, 0.41568628, 0.44313726, 0.5294118, 0.5529412, 0.64705884, 0.6666667,
                0.7647059, 0.7764706, 0.8862745,
            ],
            high_frequency_motor_speeds: vec![
                1.0,
                0.49803922,
                0.29803923,
                0.28235295,
                0.25882354,
                0.24705882,
                0.22352941,
                0.21176471,
                0.18431373,
                0.1764706,
                0.14901961,
                0.14117648,
                0.10980392,
                0.105882354,
                0.07450981,
                0.07058824,
                0.03529412,
            ],
        };
        assert_eq!(
            rounded_gamepad_rumble(&gamepad_rumble),
            rounded_gamepad_rumble(&expected_gamepad_rumble)
        );
    }

    #[test]
    fn breakpoints_close_together() {
        let gamepad_rumble = load_from_test_data("breakpoints_close_together.haptic");
        let expected_gamepad_rumble = GamepadRumble {
            durations_ms: vec![1, 1, 1, 7],
            low_frequency_motor_speeds: vec![0.0, 0.49803922, 0.24705882, 0.29803923],
            high_frequency_motor_speeds: vec![0.0, 0.0, 0.0, 0.0],
        };
        assert_eq!(
            rounded_gamepad_rumble(&gamepad_rumble),
            rounded_gamepad_rumble(&expected_gamepad_rumble)
        );
    }

    #[test]
    fn amplitude_envelope_silent() {
        let gamepad_rumble = load_from_test_data("amplitude_envelope_silent.haptic");
        let expected_gamepad_rumble = GamepadRumble {
            durations_ms: vec![25, 25, 25, 25, 50],
            low_frequency_motor_speeds: vec![0.0, 0.0, 0.0, 0.0, 0.0],
            high_frequency_motor_speeds: vec![1.0, 0.7490196, 0.49803922, 0.24705882, 0.0],
        };
        assert_eq!(
            rounded_gamepad_rumble(&gamepad_rumble),
            rounded_gamepad_rumble(&expected_gamepad_rumble)
        );
    }

    #[test]
    fn frequency_envelope_silent() {
        let gamepad_rumble = load_from_test_data("frequency_envelope_silent.haptic");
        let expected_gamepad_rumble = GamepadRumble {
            durations_ms: vec![25, 25, 25, 25, 50],
            low_frequency_motor_speeds: vec![0.0, 0.24705882, 0.49803922, 0.7490196, 1.0],
            high_frequency_motor_speeds: vec![0.0, 0.0, 0.0, 0.0, 0.0],
        };
        assert_eq!(
            rounded_gamepad_rumble(&gamepad_rumble),
            rounded_gamepad_rumble(&expected_gamepad_rumble)
        );
    }

    // Test that loading a complex and big haptic file doesn't panic.
    // This test doesn't compare the gamepad_rumble, as the data is too big
    // to write an `expected_gamepad_rumble` here.
    #[test]
    fn big_haptic() {
        let gamepad_rumble = load_from_test_data("car.vij");

        // Just make sure there is some data in gamepad_rumble
        assert!(gamepad_rumble.durations_ms.len() > 100);
    }

    // This tests various corner cases of the extend_breakpoints() function.
    #[test]
    #[rustfmt::skip]
    fn extend_breakpoints() {
        // No breakpoints
        check_extend_breakpoints(
            &[],
            &[1.0],
            &[amp(1.0, 0.0)]);

        // One breakpoint, with timings extended before, at and after that breakpoint
        check_extend_breakpoints(
            &[amp(1.0, 0.5)],
            &[0.5],
            &[amp(0.5, 0.0), amp(1.0, 0.5)]);
        check_extend_breakpoints(
            &[amp(1.0, 0.5)],
            &[1.0],
            &[amp(1.0, 0.5)]);
        check_extend_breakpoints(
            &[amp(1.0, 0.5)],
            &[1.5],
            &[amp(1.0, 0.5), amp(1.5, 0.5)]);

        // Two breakpoints, with timings extended in-between with one breakpoint
        check_extend_breakpoints(
            &[amp(1.0, 0.5), amp(2.0, 0.6)],
            &[1.5],
            &[amp(1.0, 0.5), amp(1.5, 0.55), amp(2.0, 0.6)]);

        // Two breakpoints, with timings extended in-between with two breakpoints
        check_extend_breakpoints(
            &[amp(1.0, 0.5), amp(2.0, 0.6)],
            &[1.5, 1.7],
            &[amp(1.0, 0.5), amp(1.5, 0.55), amp(1.7, 0.57), amp(2.0, 0.6)]);

        // Two breakpoints, with lots of extensions
        check_extend_breakpoints(
            &[amp(1.0, 0.5), amp(2.0, 0.6)],
            &[0.5, 1.0, 1.5, 1.7, 2.0, 2.5],
            &[amp(0.5, 0.0), amp(1.0, 0.5), amp(1.5, 0.55), amp(1.7, 0.57), amp(2.0, 0.6), amp(2.5, 0.6)]);
    }

    fn check_extend_breakpoints(
        original_breakpoints: &[AmplitudeBreakpoint],
        extended_timings: &[f32],
        expected_breakpoints: &[AmplitudeBreakpoint],
    ) {
        let actual_breakpoints = super::extend_breakpoints(original_breakpoints, extended_timings);
        let actual_breakpoints = rounded_amplitude_breakpoints(&actual_breakpoints);
        assert_eq!(actual_breakpoints, expected_breakpoints);
    }
}
