// Copyright (c) Meta Platforms, Inc. and affiliates.

use crate::{
    v1::{AmplitudeBreakpoint, Emphasis},
    waveform::Waveform,
};
use utils::test_utils::{self, rounded_f32};

/// Prints the given list of breakpoints as CSV format, so that the output can be copy & pasted
/// into a spreadsheet.
/// In the spreadsheet a graph can be used to visualize the list of breakpoints. This is useful
/// for debugging.
#[allow(dead_code)]
pub fn print_breakpoints_as_csv(breakpoints: &[AmplitudeBreakpoint], header: &str) {
    println!("---- {} ----", header);
    println!("time[s], amplitude");
    for breakpoint in breakpoints {
        println!("{},{}", breakpoint.time, breakpoint.amplitude);
    }
}

/// Like print_breakpoints_as_csv(), but for visualizing a Waveform instead of a breakpoint list.
#[allow(dead_code)]
pub fn print_waveform_as_csv(waveform: &Waveform, header: &str) {
    println!("---- {} ----", header);
    println!("time[ms], amplitude");
    let mut time_point_ms = 0;
    for i in 0..waveform.timings.len() {
        println!("{},{}", time_point_ms, waveform.amplitudes[i]);
        time_point_ms += waveform.timings[i];
        println!("{},{}", time_point_ms, waveform.amplitudes[i]);
    }
}

/// Helper to create an AmplitudeBreakpoint with rounded values
pub fn amp(time: f32, amplitude: f32) -> AmplitudeBreakpoint {
    AmplitudeBreakpoint {
        time: test_utils::rounded_f32(time, 5),
        amplitude: test_utils::rounded_f32(amplitude, 5),
        emphasis: None,
    }
}

/// Creates a Waveform from an array of tuples
pub fn create_waveform(entries: &[(i64, i32)]) -> Waveform {
    let timings = entries.iter().map(|entry| entry.0).collect();
    let amplitudes = entries.iter().map(|entry| entry.1).collect();
    Waveform {
        timings,
        amplitudes,
    }
}

/// Helper to create an AmplitudeBreakpoint with emphasis with rounded values
pub fn emp(
    time: f32,
    amplitude: f32,
    emphasis_amplitude: f32,
    emphasis_frequency: f32,
) -> AmplitudeBreakpoint {
    AmplitudeBreakpoint {
        time,
        amplitude,
        emphasis: Some(Emphasis {
            amplitude: emphasis_amplitude,
            frequency: emphasis_frequency,
        }),
    }
}

// Helper function to round amplitude breakpoints ´time´ and ´amplitude´ values using
// the function ´rounded_f32´. This is useful to avoid floating-point
// precision problems when comparing breakpoints.
pub fn rounded_amplitude_breakpoints(
    amplitude_breakpoints: &[AmplitudeBreakpoint],
) -> Vec<AmplitudeBreakpoint> {
    amplitude_breakpoints
        .iter()
        .map(|x| AmplitudeBreakpoint {
            time: rounded_f32(x.time, 5),
            amplitude: rounded_f32(x.amplitude, 5),
            emphasis: x.emphasis,
        })
        .collect::<Vec<AmplitudeBreakpoint>>()
}
