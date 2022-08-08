use crate::v1::AmplitudeBreakpoint;
use core::f32;

/// Contains parameters used by `Interpolator`

#[derive(Debug, PartialEq)]
pub struct InterpolationParameters {
    /// Quantization depth corresponds to the number of values that
    /// are available to represent an amplitude breakpoint in a Waveform. E.g.
    /// for 8 bits, there are 256 values of amplitude resolution.
    q_depth: u32,
    /// Corresponds to the minimum space between interpolated breakpoints
    min_time_step: f32,
    /// Sampling frequency of the interpolation linear space, based on
    /// min_time_step
    sampling_freq: f32,
}

impl InterpolationParameters {
    pub fn new(q_bits: u32, mut min_time_step: f32) -> Self {
        let q_depth = 2u32.pow(q_bits);

        let mut sampling_freq = 0.0;
        if min_time_step > 0.0 {
            sampling_freq = 1.0 / min_time_step;
        } else {
            min_time_step = 0.0;
        }

        Self {
            q_depth,
            min_time_step,
            sampling_freq,
        }
    }
}

// Can't use f32::clamp(), which was introduced in Rust 1.50.0. We are stuck
// with a lower version of Rust and therefore need to implement clamp() ourselves.
// This can be removed once we support Rust >= 1.50.0.
fn clamp(number: f32, min: f32, max: f32) -> f32 {
    number.min(max).max(min)
}

pub struct Interpolator {
    parameters: InterpolationParameters,
}

impl Interpolator {
    pub fn new(parameters: InterpolationParameters) -> Self {
        Self { parameters }
    }

    /// Interpolates an array of amplitude breakpoints based on InterpolationParameters.
    ///
    /// It iterates through each breakpoint and interpolates with its adjacent breakpoint to create
    /// an array with better breakpoint resolution for playback on players that don't do
    /// interpolation themselves, like Android or Unity's Gamepad. It also removes
    /// redundant amplitude breakpoints that players doesn't have resolution to play.
    pub fn process(
        &self,
        amplitude_breakpoints: &[AmplitudeBreakpoint],
    ) -> Vec<AmplitudeBreakpoint> {
        let mut previous_breakpoint: Option<&AmplitudeBreakpoint> = None;
        let mut amplitude_breakpoints_interpolated = Vec::new();

        for breakpoint in amplitude_breakpoints {
            if previous_breakpoint.is_some() {
                let interpolated_segment = self.linear_space_interpolation(
                    previous_breakpoint
                        .as_ref()
                        .map_or(0.0, |x: &&AmplitudeBreakpoint| x.time),
                    breakpoint.time,
                    previous_breakpoint
                        .as_ref()
                        .map_or(0.0, |x: &&AmplitudeBreakpoint| x.amplitude),
                    breakpoint.amplitude,
                );

                amplitude_breakpoints_interpolated.extend(
                    self.remove_redundant_amplitudes(
                        interpolated_segment.0,
                        interpolated_segment.1,
                    ),
                );
            }

            previous_breakpoint = Some(breakpoint);
        }
        amplitude_breakpoints_interpolated
    }

    /// Creates an array of linear interpolated values between time_a and time_b
    /// that area linearly spaced with 1/parameters.freq_sampling
    /// Always includes original amplitude values for time_a and time_b.
    fn linear_space_interpolation(
        &self,
        time_a: f32,
        time_b: f32,
        amp_a: f32,
        amp_b: f32,
    ) -> (Vec<f32>, Vec<f32>) {
        debug_assert!(time_b >= time_a, "time_b needs to be after time_a");
        let interval = time_b - time_a;
        let total_points = ((self.parameters.sampling_freq * interval) + 1.0) as usize;

        let mut time_result: Vec<f32> = Vec::new();
        let mut amplitude_result: Vec<f32> = Vec::new();

        // Only interpolate with 3 points in the linear space otherwise it's pointless
        // to interpolate
        if interval > self.parameters.min_time_step && total_points >= 3 {
            // TODO: Remove itertools_num dependency
            let interp_time =
                itertools_num::linspace(time_a, time_b, total_points).collect::<Vec<f32>>();

            // Interpolate for each time value of the linear space and push into
            // time and amplitude arrays
            for (time, amplitude) in interp_time.iter().zip(interp_time.iter().map(|x| {
                // Use clamp() to ensure that the time value is always inside the
                // range [time_a, time_b]. This is needed because sometimes
                // itertools_num::linspace() returns values outside of that range,
                // probably due to floating point precision issues.
                utils::interpolate(time_a, time_b, amp_a, amp_b, clamp(*x, time_a, time_b))
            })) {
                time_result.push(*time);
                amplitude_result.push(amplitude);
            }
        } else {
            // return original breakpoints if there's no interpolation to be added
            time_result.push(time_a);
            time_result.push(time_b);

            amplitude_result.push(amp_a);
            amplitude_result.push(amp_b);
        }

        (time_result, amplitude_result)
    }

    /// Returns an AmplitudeBreakpoint array without amplitude points that belong to the same
    /// "quantization bin". It removes redundant amplitude values for which players like Android
    /// or Unity's Gamepad don't have the resolution to play.
    ///
    /// A quantization bin is a value that corresponds to the
    /// following, with Q_DEPTH=256:
    ///     `bin = round(value * 256)/256`
    /// For example, 0.002, 0.003, 0.004 and 0.005 belong to the same quantization bin. Both values
    /// would result in 0.0039 (that corresponds to 1 in a Waveform -> 0.0039*256=1).
    /// On the other hand, 0.002 and 0.006 belong to different bins. 0.002 to 1, and 0.006 to 2 in
    /// a Waveform.
    /// In the first example, all amplitude values will have the same quantized value. So, there's
    /// no added value in having a breakpoint for each one of them since the player will play them all
    /// with the same amplitude. This means we only need one amplitude value to represent all of
    /// those values to be played on the player.
    ///
    /// Always includes original amplitude values for first and last position.
    fn remove_redundant_amplitudes(
        &self,
        interp_time: Vec<f32>,
        interp_amp: Vec<f32>,
    ) -> Vec<AmplitudeBreakpoint> {
        let mut time_aux = Vec::new();
        let mut amplitude_aux = Vec::new();

        let time_first = interp_time.first().unwrap();
        let time_last = interp_time.last().unwrap();

        let mut current_quantization_bin = 0.0;
        let error_margin = f32::EPSILON;

        for (time, amp) in interp_time.iter().zip(interp_amp.iter()) {
            let amp_quantized =
                (amp * (self.parameters.q_depth as f32)).round() / (self.parameters.q_depth as f32);

            // Checks if the quantized amplitude value is the same as the current quantization bin
            // if it is, the value is discarded, otherwise its added.
            // Also, make sure to add original first and last breakpoint
            if (amp_quantized - current_quantization_bin).abs() < error_margin
                && ((time - time_first).abs() > error_margin
                    && (time - time_last).abs() > error_margin)
            {
                continue;
            } else {
                time_aux.push(time);
                amplitude_aux.push(amp);
                current_quantization_bin = amp_quantized;
            }
        }

        // Convert aux time and amplitude arrays into AmplitudeBreakpoint array
        let mut amplitude_breakpoints_interpolated = Vec::new();
        for (time, amplitude) in time_aux.into_iter().zip(amplitude_aux.into_iter()) {
            amplitude_breakpoints_interpolated.push(AmplitudeBreakpoint {
                time: *time,
                amplitude: *amplitude,
                emphasis: None,
            })
        }

        amplitude_breakpoints_interpolated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{amp, rounded_amplitude_breakpoints};

    const Q_BITS: u32 = 8;
    const MIN_TIME_STEP: f32 = 0.025;

    #[test]
    /// Tests that the interpolation results in only 3 points
    fn check_linear_interpolation_3_points() {
        let interpolated_data =
            Interpolator::new(InterpolationParameters::new(Q_BITS, MIN_TIME_STEP));

        let (result_time, result_amplitude) =
            interpolated_data.linear_space_interpolation(0.0, 0.05, 0.0, 1.0);

        let expected_time = vec![0.0, 0.025, 0.05];
        let expected_amplitude = vec![0.0, 0.5, 1.0];

        assert_eq!(expected_time, result_time);
        assert_eq!(expected_amplitude, result_amplitude);
    }
    #[test]
    /// Tests that the interpolation results in 2 original points
    fn check_linear_interpolation_2_points() {
        let interpolator = Interpolator::new(InterpolationParameters::new(Q_BITS, MIN_TIME_STEP));

        // interpolating when points are less than min_step_size apart
        let (result_time, result_amplitude) =
            interpolator.linear_space_interpolation(0.01, 0.02, 1.0, 0.5);

        let expected_time = vec![0.01, 0.02];
        let expected_amplitude = vec![1.0, 0.5];

        assert_eq!(expected_time, result_time);
        assert_eq!(expected_amplitude, result_amplitude);
    }

    #[test]
    /// Check that removing redundant amplitudes doesn't discard amplitudes in diff. quantization bins
    fn check_remove_redundant_amplitudes_not_discarding() {
        let interpolator = Interpolator::new(InterpolationParameters::new(Q_BITS, MIN_TIME_STEP));

        let interp_time = vec![0.0, 1.0, 2.0];
        let interp_amp = vec![0.0, 0.004, 0.008];
        let result_removed_redundant_amplitudes =
            interpolator.remove_redundant_amplitudes(interp_time, interp_amp);

        let expected_remove_redundant_amplitudes: Vec<AmplitudeBreakpoint> = vec![
            AmplitudeBreakpoint {
                time: 0.0,
                amplitude: 0.0,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 1.0,
                amplitude: 0.004,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 2.0,
                amplitude: 0.008,
                emphasis: None,
            },
        ];

        assert_eq!(
            expected_remove_redundant_amplitudes,
            result_removed_redundant_amplitudes
        );
    }

    #[test]
    /// Check that removing redundant amplitudes discards amplitudes in the same quantization bin
    fn check_remove_redundant_amplitudes_discarding_values() {
        let interpolator = Interpolator::new(InterpolationParameters::new(Q_BITS, MIN_TIME_STEP));

        let interp_time = vec![0.0, 1.0, 2.0];
        let interp_amp = vec![0.002, 0.005, 0.006];

        let result_remove_redundant_amplitudes =
            interpolator.remove_redundant_amplitudes(interp_time, interp_amp);

        let expected_remove_redundant_amplitudes: Vec<AmplitudeBreakpoint> = vec![
            AmplitudeBreakpoint {
                time: 0.0,
                amplitude: 0.002,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 2.0,
                amplitude: 0.006,
                emphasis: None,
            },
        ];

        assert_eq!(
            expected_remove_redundant_amplitudes,
            result_remove_redundant_amplitudes
        );
    }

    #[test]
    /// Check interpolation of haptic data for a ramp up
    /// from 0 to 0.1 amplitude in 0.5 second with 3 breakpoints
    fn check_interpolator_process_ramp_up_slow_attack() {
        let input_data = vec![amp(0.0, 0.0), amp(0.25, 0.05), amp(0.5, 0.1)];

        let interpolator = Interpolator::new(InterpolationParameters::new(Q_BITS, MIN_TIME_STEP));
        let result_interpolated_data =
            rounded_amplitude_breakpoints(&interpolator.process(&input_data));

        // NOTE: this the expected data for interpolation with the following parameters:
        // Q_BITS = 8
        // min_time_step_size_secs = 0.025
        let expected_interpolated_data = vec![
            amp(0.0, 0.0),
            amp(0.025, 0.005),
            amp(0.05, 0.01),
            amp(0.075, 0.015),
            amp(0.1, 0.02),
            amp(0.125, 0.025),
            amp(0.15, 0.03),
            amp(0.175, 0.035),
            amp(0.2, 0.04),
            amp(0.225, 0.045),
            amp(0.25, 0.05),
            amp(0.25, 0.05),
            amp(0.275, 0.055),
            amp(0.3, 0.06),
            amp(0.325, 0.065),
            amp(0.35, 0.07),
            amp(0.375, 0.075),
            amp(0.4, 0.08),
            amp(0.425, 0.085),
            amp(0.45, 0.09),
            amp(0.475, 0.095),
            amp(0.5, 0.1),
        ];

        assert_eq!(expected_interpolated_data, result_interpolated_data);
    }

    // Checks interpolation of the output of emphasize().
    // The output of emphasize() has a special case: Consecutive
    // breakpoints with the same time value, but a different amplitude value.
    #[test]
    fn check_emphasized_input() {
        let emphasized_clip = vec![
            amp(0.0, 0.2),
            amp(0.1, 0.3),
            amp(0.195, 0.205),
            amp(0.195, 0.0),
            amp(0.2, 0.0),
            amp(0.2, 0.8),
            amp(0.215, 0.8),
            amp(0.215, 0.245),
            amp(0.3, 0.5),
            amp(0.5, 0.5),
        ];
        let expected_interpolated_clip = vec![
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
        let interpolator = Interpolator::new(InterpolationParameters::new(Q_BITS, MIN_TIME_STEP));
        let actual_interpolated_clip =
            rounded_amplitude_breakpoints(&interpolator.process(&emphasized_clip));
        assert_eq!(actual_interpolated_clip, expected_interpolated_clip);
    }

    #[test]
    fn check_negative_and_zero_input_interpolation_parameters() {
        let result_parameters = InterpolationParameters::new(8, -2.0);

        let expected_parameters = InterpolationParameters {
            min_time_step: 0.0,
            sampling_freq: 0.0,
            q_depth: 256,
        };
        assert_eq!(result_parameters, expected_parameters);
    }
}
