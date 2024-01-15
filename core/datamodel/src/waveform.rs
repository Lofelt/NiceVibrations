// Copyright (c) Meta Platforms, Inc. and affiliates.

use crate::v1::AmplitudeBreakpoint;

// A Waveform is a representation of a vibration pattern.
//
// Each entry of the Waveform causes a vibration of the given duration and
// amplitude.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Waveform {
    // Timings, in milliseconds
    pub timings: Vec<i64>,

    // Amplitude, from 0 to WaveformConversionParameters::max_amplitude
    pub amplitudes: Vec<i32>,
}

pub struct WaveformConversionParameters {
    pub max_amplitude: i32,
}

impl Waveform {
    /// Creates a Waveform from amplitude breakpoints
    pub fn from_breakpoints(
        breakpoints: &[AmplitudeBreakpoint],
        parameters: WaveformConversionParameters,
    ) -> Self {
        let mut timings = Vec::<i64>::new();
        let mut amplitudes = Vec::<i32>::new();
        let mut accumulated_duration: f32 = 0.0;

        // Iterate over each pair of consecutive breakpoints and create a waveform
        // entry (duration and amplitude) from the pair.
        // The duration is the time between the two breakpoints, and the amplitude
        // is the amplitude of the first breakpoint.
        for breakpoint_pair in breakpoints.windows(2) {
            let breakpoint_a = &breakpoint_pair[0];
            let breakpoint_b = &breakpoint_pair[1];
            let duration = breakpoint_b.time - breakpoint_a.time;

            if duration > 0.0 {
                // Timestamps in the DataModel are the start time / offset of a breakpoint,
                // while the timings in a Waveform are the duration of the breakpoint.
                // DataModel timestamps are in seconds, and Waveform timings are in milliseconds.
                //
                // Due to rounding down to milliseconds, a rounding error can accumulate. As soon
                // as the rounding error (timing_error_ms) is larger than 1ms, a timing
                // correction (timing_error_ms) is added to the duration to reduce the
                // rounding error.
                let timing_error_ms =
                    (breakpoint_a.time - accumulated_duration / 1000.0) * 1000.0;
                let duration_ms = ((duration * 1000.0) + timing_error_ms).round() as i64;

                if duration_ms > 0 {
                    timings.push(duration_ms);
                    accumulated_duration += duration_ms as f32;

                    // DataModel amplitudes go from 0 to 1, convert to 0 to max_amplitude
                    let amplitude =
                        (breakpoint_a.amplitude * parameters.max_amplitude as f32) as i32;
                    amplitudes.push(amplitude);
                }
            }
        }

        Self {
            timings,
            amplitudes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{self, amp};
    use std::ops::Deref;

    /// Verifies that a clip that has been emphasized and then interpolated
    /// correctly converts to a waveform.
    /// The output of emphasize() has a special case: Consecutive
    /// breakpoints with the same time value, but a different amplitude value.
    #[test]
    fn convert_interpolated_emphasized_clip() {
        let interpolated_emphasized_clip = vec![
            amp(0.0, 0.2),
            amp(0.025, 0.225),
            amp(0.05, 0.25),
            amp(0.075, 0.275),
            amp(0.1, 0.3),
            amp(0.1, 0.3),
            amp(0.13167, 0.26833),
            amp(0.16333, 0.23667),
            amp(0.195, 0.205),
            amp(0.195, 0.205),
            amp(0.195, 0.0),
            amp(0.195, 0.0),
            amp(0.2, 0.0),
            amp(0.2, 0.0),
            amp(0.2, 0.8),
            amp(0.2, 0.8),
            amp(0.215, 0.8),
            amp(0.215, 0.8),
            amp(0.215, 0.245),
            amp(0.215, 0.245),
            amp(0.24333, 0.33),
            amp(0.27167, 0.415),
            amp(0.3, 0.5),
            amp(0.3, 0.5),
            amp(0.5, 0.5),
        ];
        let expected_waveform = test_utils::create_waveform(&[
            (25, 51),
            (25, 57),
            (25, 63),
            (25, 70),
            (32, 76),
            (31, 68),
            (32, 60),
            (5, 0),
            (15, 204),
            (28, 62),
            (29, 84),
            (28, 105),
            (200, 127),
        ]);
        let actual_waveform = Waveform::from_breakpoints(
            interpolated_emphasized_clip.deref(),
            WaveformConversionParameters { max_amplitude: 255 },
        );
        assert_eq!(expected_waveform, actual_waveform);
    }

    // This tests that the output of two_emphasis_breakpoints_close_full() can
    // be converted to a waveform.
    //
    // The output has a special case of 3 breakpoints at the same time.
    #[test]
    fn convert_zero_duration_breakpoints() {
        let emphasized_clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            amp(0.16, 0.23333),
            amp(0.16, 0.00392),
            amp(0.19, 0.00392),
            amp(0.19, 0.9),
            amp(0.22, 0.9),
            amp(0.22, 0.36667),
            amp(0.22, 0.8),
            amp(0.24, 0.8),
            amp(0.24, 0.3),
            amp(0.3, 0.1),
            amp(0.4, 0.0),
        ];
        let expected_waveform = test_utils::create_waveform(&[
            (100, 0),
            (60, 25),
            (30, 0),
            (30, 229),
            (20, 204),
            (60, 76),
            (100, 25),
        ]);
        let actual_waveform = Waveform::from_breakpoints(
            emphasized_clip.deref(),
            WaveformConversionParameters { max_amplitude: 255 },
        );
        assert_eq!(expected_waveform, actual_waveform);
    }

    // This tests that breakpoints close together use proper rounding and don't
    // create waveform entries of 0ms.
    #[test]
    fn close_breakpoints_rounding() {
        let breakpoints = [
            amp(0.0, 0.0),
            amp(0.001, 0.2),
            amp(0.002, 0.0),
            amp(0.003, 0.2),
            amp(0.004, 0.0),
        ];
        let actual_waveform = Waveform::from_breakpoints(
            &breakpoints,
            WaveformConversionParameters { max_amplitude: 255 },
        );
        let expected_waveform = test_utils::create_waveform(&[(1, 0), (1, 51), (1, 0), (1, 51)]);
        assert_eq!(expected_waveform, actual_waveform);
    }
}
