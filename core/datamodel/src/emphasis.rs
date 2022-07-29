use crate::v1::{AmplitudeBreakpoint, Emphasis};
use std::time::Duration;

pub struct EmphasisParameters {
    pub ducking_before_length: Duration,
    pub ducking_after_length: Duration,
    pub emphasis_length: Duration,
    pub ducking_amplitude: f32,
}

// We ignore the amplitude of the emphasis here, and always use the highest
// possible amplitude, 1.0. This is done so that the emulated emphasis is
// more distinct, especially when the surrounding continuous amplitude envelope
// is already at a high amplitude.
const EMPHASIS_AMPLITUDE: f32 = 1.0;

impl Default for EmphasisParameters {
    fn default() -> Self {
        Self {
            ducking_before_length: Duration::from_millis(30),
            emphasis_length: Duration::from_millis(30),
            ducking_after_length: Duration::from_millis(30),
            ducking_amplitude: 0.0,
        }
    }
}

/// Renders the emphasis of breakpoints into the continuous amplitude signal.
///
/// Some systems like Android and Unity's Gamepad do not have support for
/// transients, therefore emphasis needs to be simulated by modifying the
/// continuous amplitude signal instead.
///
/// This is just a wrapper around Emphasizer, see Emphasizer for more details.
pub fn emphasize(
    amplitude_breakpoints: &[AmplitudeBreakpoint],
    parameters: EmphasisParameters,
) -> Vec<AmplitudeBreakpoint> {
    let mut emphasizer = Emphasizer::new(parameters, amplitude_breakpoints);
    emphasizer.process();
    emphasizer.result()
}

/// Renders the emphasis of breakpoints into the continuous amplitude signal.
///
/// To render a breakpoint with emphasis, the following is done:
/// 1. The amplitude of the continuous signal is set to 0 for a short time _before_
///    the breakpoint with emphasis.
///    This is called "ducking before" here. This is done so that there is a bigger and
///    more sudden change of amplitude at the breakpoint with emphasis, which more
///    closely resembles a transient.
///    The length of the ducking before can be controlled with
///    EmphasisParameters::ducking_before_length.
/// 2. The amplitude of the continuous signal is set to 1.0 for a short time after
///    the breakpoint with emphasis.
///    Without this, the emphasis would have a duration of zero, which is not supported
///    by Waveform.
///    The length of the emphasis can be controlled with EmphasisParameters::emphasis_length.
/// 3. The amplitude of the continuous signal is set to 0 for a short time _after_
///    the emphasis.
///    This is called "ducking after" here. This is done so that the emphasis feels distinct
///    from the rest of the continuous signal.
///    The length of the ducking after can be controlled with
///    EmphasisParameters::ducking_after_length.
///
/// The above algorithm is implemented in process().
///
struct Emphasizer<'bps> {
    parameters: EmphasisParameters,
    amplitude_breakpoints: &'bps [AmplitudeBreakpoint],
    result: Vec<AmplitudeBreakpoint>,
}

impl<'bps> Emphasizer<'bps> {
    pub fn new(
        parameters: EmphasisParameters,
        amplitude_breakpoints: &'bps [AmplitudeBreakpoint],
    ) -> Self {
        Self {
            parameters,
            amplitude_breakpoints,
            result: Vec::new(),
        }
    }

    pub fn result(self) -> Vec<AmplitudeBreakpoint> {
        self.result
    }

    // Iterates over all breakpoints and renders the emphasis by appending new and
    // transformed breakpoints to self.result.
    pub fn process(&mut self) {
        let mut next_emphasis = self
            .amplitude_breakpoints
            .iter()
            .find(|breakpoint| breakpoint.emphasis.is_some());
        let mut prev_emphasis: Option<&AmplitudeBreakpoint> = None;
        for (index, breakpoint) in self.amplitude_breakpoints.iter().enumerate() {
            match breakpoint.emphasis {
                None => {
                    self.process_normal_breakpoint(breakpoint, prev_emphasis, next_emphasis);
                }
                Some(emphasis) => {
                    self.process_emphasis_breakpoint(breakpoint, index, emphasis);
                    prev_emphasis = next_emphasis;
                    next_emphasis = self.amplitude_breakpoints[index + 1..]
                        .iter()
                        .find(|breakpoint| breakpoint.emphasis.is_some());
                }
            }
        }
    }

    // A normal breakpoint is either appended to self.result or skipped.
    // It is skipped if the breakpoint is within a ducking area or an emphasis
    // area, i.e. if it is either closely before or closely after a breakpoint
    // with emphasis.
    fn process_normal_breakpoint(
        &mut self,
        breakpoint: &AmplitudeBreakpoint,
        prev_emphasis: Option<&AmplitudeBreakpoint>,
        next_emphasis: Option<&AmplitudeBreakpoint>,
    ) {
        let skip_due_to_ducking_before = match next_emphasis {
            None => false,
            Some(next_emphasis) => {
                let ducking_before_start = (next_emphasis.time
                    - self.parameters.ducking_before_length.as_secs_f32())
                .max(0.0);
                let ducking_before_range = ducking_before_start..=next_emphasis.time;
                ducking_before_range.contains(&breakpoint.time)
            }
        };
        let skip_due_to_emphasis_and_ducking_after = match prev_emphasis {
            None => false,
            Some(prev_emphasis) => {
                let emphasis_end =
                    prev_emphasis.time + self.parameters.emphasis_length.as_secs_f32();
                let ducking_after_end =
                    emphasis_end + self.parameters.ducking_after_length.as_secs_f32();
                let range = prev_emphasis.time..=ducking_after_end;
                range.contains(&breakpoint.time)
            }
        };
        let skip = skip_due_to_ducking_before || skip_due_to_emphasis_and_ducking_after;
        if !skip {
            self.result.push(breakpoint.clone());
        }
    }

    // For a breakpoint with emphasis, new breakpoints for the ducking areas (before and after)
    // and for the emphasis area are appended to self.result.
    fn process_emphasis_breakpoint(
        &mut self,
        emphasis_breakpoint: &AmplitudeBreakpoint,
        emphasis_index: usize,
        emphasis: Emphasis,
    ) {
        self.process_ducking_before_area(emphasis_breakpoint, emphasis_index);
        self.process_emphasis_and_ducking_after_area(emphasis_breakpoint, emphasis_index, emphasis);
    }

    // Appends the breakpoints of the ducking before area to self.result.
    //
    // The ducking before area has up to 3 breakpoints:
    // 1. A breakpoint at the start of ducking before, with the amplitude the continuous
    //    amplitude signal would have had normally
    // 2. A breakpoint at the start of ducking before, with amplitude 0
    // 3. A breakpoint at the end of ducking before, with amplitude 0
    fn process_ducking_before_area(
        &mut self,
        emphasis_breakpoint: &AmplitudeBreakpoint,
        emphasis_index: usize,
    ) {
        let last_time = match self.result.last() {
            Some(last) => last.time,
            None => 0.0,
        };
        if emphasis_breakpoint.time <= last_time {
            return;
        }
        let ducking_before_start = (emphasis_breakpoint.time
            - self.parameters.ducking_before_length.as_secs_f32())
        .max(0.0)
        .max(last_time);
        let index_before_ducking_before = self.amplitude_breakpoints[..emphasis_index]
            .iter()
            .rposition(|breakpoint| breakpoint.time < ducking_before_start);

        // Breakpoint 1: Start of ducking before
        // The amplitude needs to be interpolated from the breakpoint before and after that point.
        if let Some(index_before_ducking_before) = index_before_ducking_before {
            debug_assert!(index_before_ducking_before + 1 < self.amplitude_breakpoints.len());
            let breakpoint_before_ducking_before =
                &self.amplitude_breakpoints[index_before_ducking_before];
            let breakpoint_in_ducking_before =
                &self.amplitude_breakpoints[index_before_ducking_before + 1];
            debug_assert!((breakpoint_before_ducking_before.time
                ..=breakpoint_in_ducking_before.time)
                .contains(&ducking_before_start));
            let breakpoint_at_ducking_before_start =
                AmplitudeBreakpoint::from_interpolated_breakpoints(
                    breakpoint_before_ducking_before,
                    breakpoint_in_ducking_before,
                    ducking_before_start,
                );

            self.result.push(breakpoint_at_ducking_before_start);
        }

        // Breakpoint 2: Start of ducking before, amplitude 0
        self.result.push(AmplitudeBreakpoint {
            time: ducking_before_start,
            amplitude: self.parameters.ducking_amplitude,
            emphasis: None,
        });

        // Breakpoint 3: End of ducking before, amplitude 0
        self.result.push(AmplitudeBreakpoint {
            time: emphasis_breakpoint.time,
            amplitude: self.parameters.ducking_amplitude,
            emphasis: None,
        });
    }

    // Appends the breakpoints of the emphasis area and the ducking after area
    // to self.result.
    //
    // The emphasis and ducking after areas have up to 5 breakpoints:
    // 1. A breakpoint at the start of emphasis, with amplitude 1.0
    // 2. A breakpoint at the end of emphasis, with amplitude 1.0
    // 3. A breakpoint at the start of ducking after, with amplitude 0.0
    // 4. A breakpoint at the end of ducking after, with amplitude 0.0
    // 5. A breakpoint at the end of ducking after, with the amplitude the continuous
    //    amplitude signal would have had normally
    fn process_emphasis_and_ducking_after_area(
        &mut self,
        emphasis_breakpoint: &AmplitudeBreakpoint,
        emphasis_index: usize,
        _emphasis: Emphasis,
    ) {
        let last_time = match self.result.last() {
            Some(last) => last.time,
            None => 0.0,
        };

        let emphasis_start = emphasis_breakpoint.time.max(last_time);
        let emphasis_end = (emphasis_breakpoint.time
            + self.parameters.emphasis_length.as_secs_f32())
        .max(last_time);

        // If the emphasis has a duration of 0ms, return right away without adding
        // any emphasis or ducking after.
        // This case can happen if the emphasis falls completely into the ducking
        // after range of the previous emphasis breakpoint.
        if emphasis_end - emphasis_start <= f32::EPSILON {
            return;
        }

        // Breakpoint 1: Start of emphasis, amplitude 1.0
        self.result.push(AmplitudeBreakpoint {
            time: emphasis_start,
            amplitude: EMPHASIS_AMPLITUDE,
            emphasis: None,
        });

        // Breakpoint 2: End of emphasis, amplitude 1.0
        self.result.push(AmplitudeBreakpoint {
            time: emphasis_end,
            amplitude: EMPHASIS_AMPLITUDE,
            emphasis: None,
        });

        // Don't bother adding the ducking after breakpoints if this emphasis breakpoint
        // is the last breakpoint of the clip. The motor will be turned off after the clip
        // ends anyway.
        if emphasis_index == self.amplitude_breakpoints.len() - 1 {
            return;
        }

        // Breakpoint 3: Start of ducking after, amplitude 0
        let ducking_after_start = emphasis_end;
        self.result.push(AmplitudeBreakpoint {
            time: ducking_after_start,
            amplitude: self.parameters.ducking_amplitude,
            emphasis: None,
        });

        // Breakpoint 4: End of ducking after, amplitude 0
        let ducking_after_end =
            ducking_after_start + self.parameters.ducking_after_length.as_secs_f32();
        self.result.push(AmplitudeBreakpoint {
            time: ducking_after_end,
            amplitude: self.parameters.ducking_amplitude,
            emphasis: None,
        });

        // Breakpoint 5: End of ducking after
        // The amplitude needs to be interpolated from the breakpoint before and after that point.
        if emphasis_index < self.amplitude_breakpoints.len() - 1 {
            let start_index = emphasis_index + 1;
            let index_after_ducking_after = self.amplitude_breakpoints[start_index..]
                .iter()
                .position(|breakpoint| breakpoint.time > ducking_after_end)
                .map(|index| index + start_index);
            if let Some(index_after_ducking_after) = index_after_ducking_after {
                debug_assert!(index_after_ducking_after > emphasis_index);
                let breakpoint_after_ducking_after =
                    &self.amplitude_breakpoints[index_after_ducking_after];
                let breakpoint_in_emphasis_or_ducking_after =
                    &self.amplitude_breakpoints[index_after_ducking_after - 1];
                debug_assert!((breakpoint_in_emphasis_or_ducking_after.time
                    ..=breakpoint_after_ducking_after.time)
                    .contains(&ducking_after_end));
                let breakpoint_at_ducking_after_end =
                    AmplitudeBreakpoint::from_interpolated_breakpoints(
                        breakpoint_in_emphasis_or_ducking_after,
                        breakpoint_after_ducking_after,
                        ducking_after_end,
                    );
                self.result.push(breakpoint_at_ducking_after_end);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        emphasis::{emphasize, EmphasisParameters},
        latest_from_json,
        test_utils::emp,
        test_utils::{amp, rounded_amplitude_breakpoints},
        v1::DataModel,
        Validation,
    };
    use std::{fs, path::Path, time::Duration};

    // Checks that a clip without any emphasis at all doesn't get modified
    #[test]
    fn no_emphasis() {
        let clip = vec![amp(0.0, 0.5), amp(0.2, 0.2), amp(0.3, 0.3), amp(0.4, 0.4)];
        let emphasized_clip = emphasize(&clip, Default::default());
        assert_eq!(clip, emphasized_clip);
    }

    // Checks that a simple clip gets emphasized correctly
    #[test]
    fn simple_emphasis() {
        let clip = vec![
            amp(0.0, 0.2),
            amp(0.1, 0.3),
            emp(0.2, 0.2, 0.8, 0.7),
            amp(0.3, 0.5),
            amp(0.5, 0.5),
        ];

        let actual_emphasized_clip = rounded_amplitude_breakpoints(&emphasize(
            &clip,
            EmphasisParameters {
                ducking_before_length: Duration::from_millis(5),
                emphasis_length: Duration::from_millis(15),
                ducking_after_length: Duration::from_millis(5),
                ducking_amplitude: 1.1 / 255.0,
            },
        ));

        let expected_emphasized_clip = vec![
            amp(0.0, 0.2),
            amp(0.1, 0.3),
            amp(0.195, 0.205),
            amp(0.195, 0.00431),
            amp(0.2, 0.00431),
            amp(0.2, 1.0),
            amp(0.215, 1.0),
            amp(0.215, 0.00431),
            amp(0.22, 0.00431),
            amp(0.22, 0.26),
            amp(0.3, 0.5),
            amp(0.5, 0.5),
        ];
        assert_eq!(actual_emphasized_clip, expected_emphasized_clip);
    }

    // Checks the emphasis algorithm on a clip in which breakpoints are close to the
    // breakpoint with emphasis. This verifies that those close breakpoints get correctly
    // replaced.
    #[test]
    fn breakpoints_close_to_emphasis() {
        let clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            amp(0.195, 0.4), //< Breakpoint inside of ducking area before emphasis
            emp(0.2, 0.2, 0.8, 0.7),
            amp(0.205, 0.4), //< Breakpoint inside of emphasis area
            amp(0.217, 0.7), //< Breakpoint inside of ducking area after emphasis
            amp(0.3, 0.2),
            amp(0.4, 0.0),
        ];

        let actual_emphasized_clip = rounded_amplitude_breakpoints(&emphasize(
            &clip,
            EmphasisParameters {
                ducking_before_length: Duration::from_millis(5),
                emphasis_length: Duration::from_millis(15),
                ducking_after_length: Duration::from_millis(5),
                ducking_amplitude: 1.1 / 255.0,
            },
        ));

        let expected_emphasized_clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            amp(0.195, 0.4),
            amp(0.195, 0.4),
            amp(0.195, 0.00431),
            amp(0.2, 0.00431),
            amp(0.2, 1.0),
            amp(0.215, 1.0),
            amp(0.215, 0.00431),
            amp(0.22, 0.00431),
            amp(0.22, 0.68193),
            amp(0.3, 0.2),
            amp(0.4, 0.0),
        ];
        assert_eq!(actual_emphasized_clip, expected_emphasized_clip);
    }

    // Checks that two emphasis breakpoints really close to each other get handled
    // correctly.
    #[test]
    fn two_emphasis_breakpoints_close_1() {
        let clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            emp(0.190, 0.3, 0.9, 0.7),
            emp(0.210, 0.4, 0.8, 0.7),
            amp(0.3, 0.1),
            amp(0.4, 0.0),
        ];
        let actual_emphasized_clip = rounded_amplitude_breakpoints(&emphasize(
            &clip,
            EmphasisParameters {
                ducking_before_length: Duration::from_millis(30),
                emphasis_length: Duration::from_millis(30),
                ducking_after_length: Duration::from_millis(30),
                ducking_amplitude: 1.1 / 255.0,
            },
        ));
        let expected_emphasized_clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            amp(0.16, 0.23333),
            amp(0.16, 0.00431),
            amp(0.19, 0.00431),
            amp(0.19, 1.0),
            amp(0.22, 1.0),
            amp(0.22, 0.00431),
            amp(0.25, 0.00431),
            amp(0.25, 0.26667),
            amp(0.3, 0.1),
            amp(0.4, 0.0),
        ];
        assert_eq!(actual_emphasized_clip, expected_emphasized_clip);
    }

    // A variation of two_emphasis_breakpoints_close_1(), with slightly different
    // timing
    #[test]
    fn two_emphasis_breakpoints_close_2() {
        let clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            emp(0.190, 0.3, 0.9, 0.7),
            emp(0.230, 0.4, 0.8, 0.7),
            amp(0.3, 0.1),
            amp(0.4, 0.0),
        ];
        let actual_emphasized_clip = rounded_amplitude_breakpoints(&emphasize(
            &clip,
            EmphasisParameters {
                ducking_before_length: Duration::from_millis(5),
                emphasis_length: Duration::from_millis(30),
                ducking_after_length: Duration::from_millis(5),
                ducking_amplitude: 1.1 / 255.0,
            },
        ));
        let expected_emphasized_clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            amp(0.185, 0.28889),
            amp(0.185, 0.00431),
            amp(0.19, 0.00431),
            amp(0.19, 1.0),
            amp(0.22, 1.0),
            amp(0.22, 0.00431),
            amp(0.225, 0.00431),
            // ### The spike at 0.3875 here is incorrect. Since it's 0ms long, it will
            // ### get ignored in Waveform::from_processed_breakpoints() though, so
            // ### the spike causes no harm.
            amp(0.225, 0.3875),
            amp(0.225, 0.3875),
            amp(0.225, 0.00431),
            amp(0.23, 0.00431),
            amp(0.23, 1.0),
            amp(0.26, 1.0),
            amp(0.26, 0.00431),
            amp(0.265, 0.00431),
            amp(0.265, 0.25),
            amp(0.3, 0.1),
            amp(0.4, 0.0),
        ];
        assert_eq!(actual_emphasized_clip, expected_emphasized_clip);
    }

    // A variation of two_emphasis_breakpoints_close_1(), with slightly different
    // timing
    #[test]
    fn two_emphasis_breakpoints_close_3() {
        let clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            emp(0.190, 0.3, 0.9, 0.7),
            emp(0.230, 0.4, 0.8, 0.7),
            amp(0.3, 0.1),
            amp(0.4, 0.0),
        ];
        let actual_emphasized_clip = rounded_amplitude_breakpoints(&emphasize(
            &clip,
            EmphasisParameters {
                ducking_before_length: Duration::from_millis(5),
                emphasis_length: Duration::from_millis(30),
                ducking_after_length: Duration::from_millis(20),
                ducking_amplitude: 1.1 / 255.0,
            },
        ));
        let expected_emphasized_clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.1),
            amp(0.185, 0.28889),
            amp(0.185, 0.00431),
            amp(0.19, 0.00431),
            amp(0.19, 1.0),
            amp(0.22, 1.0),
            amp(0.22, 0.00431),
            amp(0.24, 0.00431),
            amp(0.24, 0.35714),
            amp(0.24, 1.0),
            amp(0.26, 1.0),
            amp(0.26, 0.00431),
            amp(0.28, 0.00431),
            amp(0.28, 0.18571),
            amp(0.3, 0.1),
            amp(0.4, 0.0),
        ];
        assert_eq!(actual_emphasized_clip, expected_emphasized_clip);
    }

    // Tests that multiple emphasis breakpoints close to each other get rendered correctly.
    // In this case, some emphasis breakpoints are skipped as there is not enough space to
    // render them.
    // Reduced testcase for PD-3104.
    #[test]
    fn nine_emphasis_breakpoints_close() {
        let clip = vec![
            amp(0.0, 0.0),
            emp(0.11, 0.4, 0.9, 0.7),
            emp(0.12, 0.4, 0.9, 0.7),
            emp(0.13, 0.4, 0.9, 0.7),
            emp(0.14, 0.4, 0.9, 0.7),
            emp(0.15, 0.4, 0.9, 0.7),
            emp(0.16, 0.4, 0.9, 0.7),
            emp(0.17, 0.4, 0.9, 0.7),
            emp(0.18, 0.4, 0.9, 0.7),
            emp(0.19, 0.4, 0.9, 0.7),
            amp(0.3, 0.0),
        ];
        let actual_emphasized_clip = rounded_amplitude_breakpoints(&emphasize(
            &clip,
            EmphasisParameters {
                ducking_before_length: Duration::from_millis(10),
                emphasis_length: Duration::from_millis(10),
                ducking_after_length: Duration::from_millis(10),
                ducking_amplitude: 1.1 / 255.0,
            },
        ));
        let expected_emphasized_clip = vec![
            amp(0.0, 0.0),
            amp(0.1, 0.36364),
            amp(0.1, 0.00431),
            amp(0.11, 0.00431),
            amp(0.11, 1.0),
            amp(0.12, 1.0),
            amp(0.12, 0.00431),
            amp(0.13, 0.00431),
            amp(0.13, 0.4),
            amp(0.13, 1.0),
            amp(0.14, 1.0),
            amp(0.14, 0.00431),
            amp(0.15, 0.00431),
            amp(0.15, 0.4),
            amp(0.15, 1.0),
            amp(0.16, 1.0),
            amp(0.16, 0.00431),
            amp(0.17, 0.00431),
            amp(0.17, 0.4),
            amp(0.17, 1.0),
            amp(0.18, 1.0),
            amp(0.18, 0.00431),
            amp(0.19, 0.00431),
            amp(0.19, 0.4),
            amp(0.19, 1.0),
            amp(0.2, 1.0),
            amp(0.2, 0.00431),
            amp(0.21, 0.00431),
            amp(0.21, 0.32727),
            amp(0.3, 0.0),
        ];
        assert_eq!(actual_emphasized_clip, expected_emphasized_clip);
    }

    #[test]
    fn emphasis_on_first_breakpoint() {
        let clip = vec![emp(0.0, 0.3, 0.9, 0.7), amp(0.1, 0.2), amp(0.2, 0.0)];
        let actual_emphasized_clip = rounded_amplitude_breakpoints(&emphasize(
            &clip,
            EmphasisParameters {
                ducking_before_length: Duration::from_millis(30),
                emphasis_length: Duration::from_millis(5),
                ducking_after_length: Duration::from_millis(30),
                ducking_amplitude: 1.1 / 255.0,
            },
        ));
        let expected_emphasized_clip = vec![
            amp(0.0, 1.0),
            amp(0.005, 1.0),
            amp(0.005, 0.00431),
            amp(0.035, 0.00431),
            amp(0.035, 0.265),
            amp(0.1, 0.2),
            amp(0.2, 0.0),
        ];
        assert_eq!(actual_emphasized_clip, expected_emphasized_clip);
    }

    #[test]
    fn emphasis_on_last_breakpoint() {
        let clip = vec![amp(0.0, 0.3), amp(0.1, 0.2), emp(0.2, 0.4, 0.9, 0.7)];
        let actual_emphasized_clip = rounded_amplitude_breakpoints(&emphasize(
            &clip,
            EmphasisParameters {
                ducking_before_length: Duration::from_millis(30),
                emphasis_length: Duration::from_millis(5),
                ducking_after_length: Duration::from_millis(30),
                ducking_amplitude: 1.1 / 255.0,
            },
        ));
        let expected_emphasized_clip = vec![
            amp(0.0, 0.3),
            amp(0.1, 0.2),
            amp(0.17, 0.34),
            amp(0.17, 0.00431),
            amp(0.2, 0.00431),
            amp(0.2, 1.0),
            amp(0.205, 1.0),
        ];
        assert_eq!(actual_emphasized_clip, expected_emphasized_clip);
    }

    // Loads and emphasized all .haptic files in test_data/, to check that the
    // processing doesn't panic and that the result is still a valid haptic clip.
    #[test]
    fn process_all_test_files() {
        let test_data_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/test_data");
        let dir_entries = fs::read_dir(&test_data_path).unwrap();
        for dir_entry in dir_entries {
            let dir_entry = dir_entry.unwrap();
            if !dir_entry.file_type().unwrap().is_file() {
                continue;
            }
            if dir_entry.path().extension().unwrap().to_str().unwrap() != "haptic" {
                continue;
            }

            let clip_json = std::fs::read_to_string(test_data_path.join(dir_entry.path())).unwrap();
            let clip_result = latest_from_json(&clip_json);

            // Skip invalid .haptic files
            if clip_result.is_err() {
                continue;
            }

            let clip = clip_result.unwrap().1;
            let amplitude_breakpoints = clip.signals.continuous.envelopes.amplitude;

            let emphasized_amplitude_breakpoints =
                emphasize(&amplitude_breakpoints, Default::default());
            let mut emphasized_clip: DataModel = Default::default();
            emphasized_clip.signals.continuous.envelopes.amplitude =
                emphasized_amplitude_breakpoints;
            emphasized_clip.validate().unwrap();
        }
    }
}
