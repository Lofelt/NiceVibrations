// Copyright (c) Meta Platforms, Inc. and affiliates.

#![cfg(test)]

use crate::{
    haptic_event_provider::{AmplitudeEvent, Event, FrequencyEvent, HapticEventProvider},
    streaming::Callbacks,
    streaming::Player,
    PreAuthoredClipPlayback,
};
use datamodel::v1::{DataModel, Emphasis};
use env_logger::{Builder, Env};
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
    time::Instant,
};
use utils::assert_near;
use utils::test_utils::rounded_f32;

// Both in seconds
const MAX_MAX_TIMING_ERROR: f32 = 0.010;
const MAX_AVG_TIMING_ERROR: f32 = 0.005;

// Tests that depend on the scheduler timing are disabled by default, as they don't run reliably
// on the CI. The CI machines have a high load and/or are not powerful, causing high scheduling
// variations.
// Therefore it's best to run the tests locally only, by setting this variable to true.
// Note that to see the log output for a test that passes, you need to pass --nocapture,
// like this: cargo test -- streaming --nocapture
pub const ENABLE_TIMING_DEPENDENT_TESTS: bool = false;

// Returns the clip length, which is the length of the longest envelope
pub fn clip_length(clip: &DataModel) -> Duration {
    let amp_last = clip.signals.continuous.envelopes.amplitude.last();
    let freq_last = if let Some(frequency) = clip.signals.continuous.envelopes.frequency.as_ref() {
        frequency.last()
    } else {
        None
    };

    let amp_last = amp_last.map(|breakpoint| breakpoint.time).unwrap_or(0.0);
    let freq_last = freq_last.map(|breakpoint| breakpoint.time).unwrap_or(0.0);
    if amp_last >= freq_last {
        Duration::from_secs_f32(amp_last)
    } else {
        Duration::from_secs_f32(freq_last)
    }
}

pub fn load_file_from_test_data(path: &str) -> DataModel {
    let clip = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/test_data")
            .join(path),
    )
    .unwrap();
    datamodel::latest_from_json(&clip).unwrap().1
}

// Similar to rounded_f32, only that it rounds all fields in an Event
fn rounded_event(mut event: Event, decimal_places: u32) -> Event {
    match &mut event {
        Event::Amplitude(e) => {
            e.amplitude = rounded_f32(e.amplitude, decimal_places);
            e.time = rounded_f32(e.time, decimal_places);
            e.duration = rounded_f32(e.duration, decimal_places);
        }
        Event::Frequency(e) => {
            e.frequency = rounded_f32(e.frequency, decimal_places);
            e.time = rounded_f32(e.time, decimal_places);
            e.duration = rounded_f32(e.duration, decimal_places);
        }
    }
    event
}

pub fn rounded_events(events: &[Event], decimal_places: u32) -> Vec<Event> {
    events
        .iter()
        .map(|event| rounded_event(*event, decimal_places))
        .collect()
}

// Helper to create an amplitude Event
pub fn amp(time: f32, duration: f32, value: f32) -> Event {
    Event::Amplitude(AmplitudeEvent {
        time: rounded_f32(time, 5),
        duration: rounded_f32(duration, 5),
        amplitude: rounded_f32(value, 5),
        emphasis: Emphasis {
            amplitude: f32::NAN,
            frequency: f32::NAN,
        },
    })
}

// Helper to create an amplitude Event that contains emphasis
pub fn emp(
    time: f32,
    duration: f32,
    value: f32,
    emphasis_amplitude: f32,
    emphasis_frequency: f32,
) -> Event {
    Event::Amplitude(AmplitudeEvent {
        time: rounded_f32(time, 5),
        duration: rounded_f32(duration, 5),
        amplitude: rounded_f32(value, 5),
        emphasis: Emphasis {
            amplitude: rounded_f32(emphasis_amplitude, 5),
            frequency: rounded_f32(emphasis_frequency, 5),
        },
    })
}

// Helper to create a frequency Event
pub fn freq(time: f32, duration: f32, value: f32) -> Event {
    Event::Frequency(FrequencyEvent {
        time: rounded_f32(time, 5),
        duration: rounded_f32(duration, 5),
        frequency: rounded_f32(value, 5),
    })
}

// Records and returns all events that the HapticEventProvider provides with
// get_next_event(), up to a maximum of max_events.
// Verifies some invariants along the way.
pub fn gather_events_from_provider(
    provider: &mut HapticEventProvider,
    max_events: Option<usize>,
) -> Vec<Event> {
    let mut result = Vec::new();
    loop {
        if let Some(max_events) = max_events {
            if result.len() >= max_events {
                break;
            }
        }

        let time = &provider.peek_event_start_time();
        let event = provider.get_next_event();

        // If there is an event queued up, it should be possible to get it and to peek at it
        assert_eq!(time.is_some(), event.is_some());

        if let Some(event) = event {
            // peek_event_start_time() should return the same time as get_next_event()
            assert_near!(time.unwrap(), event.time(), f32::EPSILON);
            result.push(rounded_event(event, 5));
        } else {
            break;
        }
    }
    result
}

// Loads the haptic clip from the given file into a HapticEventProvider and
// records and returns all events that the HapticEventProvider provides with
// get_next_event().
pub fn record_events_from_provider(clip_filename: &str) -> Vec<Event> {
    let clip = load_file_from_test_data(clip_filename);
    let mut provider = HapticEventProvider::new(clip);
    gather_events_from_provider(&mut provider, None)
}

struct RecordingData {
    start_time: Instant,
    events: Vec<Event>,
    errors: Vec<f32>, // in seconds
}

// Records events that a Player provides, together with the timing errors.
//
// When the player invokes the callbacks, the event passed to the callbacks is recorded
// into recording_data.
pub struct PlayerEventRecorder {
    recording_data: Arc<Mutex<RecordingData>>,
    player: Player,
}

impl PlayerEventRecorder {
    pub fn new() -> PlayerEventRecorder {
        let recording_data = Arc::new(Mutex::new(RecordingData {
            start_time: Instant::now(),
            events: Vec::new(),
            errors: Vec::new(),
        }));

        let recording_data_for_thread = recording_data.clone();
        let amplitude_event_callback = move |event: AmplitudeEvent| {
            let mut recording_data = recording_data_for_thread.lock().unwrap();
            let now = Instant::now();

            // If now is less than the recording start time (which can happen when seeking to a
            // negative time), then we can ignore the event.
            if now >= recording_data.start_time {
                let actual_time_since_start = (now - recording_data.start_time).as_secs_f32();
                let error = actual_time_since_start - event.time;
                recording_data.errors.push(error);
                recording_data
                    .events
                    .push(rounded_event(Event::Amplitude(event), 5));
            }
        };
        let recording_data_for_thread = recording_data.clone();
        let frequency_event_callback = move |event: FrequencyEvent| {
            let mut recording_data = recording_data_for_thread.lock().unwrap();
            let actual_time_since_start =
                (Instant::now() - recording_data.start_time).as_secs_f32();
            let error = actual_time_since_start - event.time;
            recording_data.errors.push(error);
            recording_data
                .events
                .push(rounded_event(Event::Frequency(event), 5));
        };

        (*recording_data.lock().unwrap()).start_time = Instant::now();
        let callbacks = Callbacks {
            amplitude_event: Box::new(amplitude_event_callback),
            frequency_event: Box::new(frequency_event_callback),
            init_thread: Box::new(|| {}),
        };
        let player = Player::new(callbacks).unwrap();
        PlayerEventRecorder {
            recording_data,
            player,
        }
    }

    pub fn recorded_events(&self) -> Vec<Event> {
        let recording_data = self.recording_data.lock().unwrap();
        recording_data.events.clone()
    }

    pub fn recorded_errors(&self) -> Vec<f32> {
        let recording_data = self.recording_data.lock().unwrap();
        recording_data.errors.clone()
    }

    pub fn clear_recording_data(&mut self, start_time_offset: f32) {
        let mut recording_data = self.recording_data.lock().unwrap();
        recording_data.events.clear();
        recording_data.errors.clear();
        recording_data.start_time = if start_time_offset >= 0.0 {
            Instant::now() - Duration::from_secs_f32(start_time_offset)
        } else {
            // A negative start time means that playback will start in the future
            Instant::now() + Duration::from_secs_f32(-start_time_offset)
        }
    }

    pub fn player(&mut self) -> &mut Player {
        &mut self.player
    }
}

pub fn print_timing_errors(recorder: &mut PlayerEventRecorder, clip_filename: &str) {
    if ENABLE_TIMING_DEPENDENT_TESTS {
        let mut errors = recorder.recorded_errors();
        if errors.is_empty() {
            return;
        }

        errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min = *errors.first().unwrap();
        let max = *errors.last().unwrap();
        let avg = errors.iter().sum::<f32>() / (errors.len() as f32);

        // Positive values mean that the events arrive later than designed, negative
        // values mean that the events arrive earlier than designed.
        log::debug!(
            "Timing errors for {}: Min: {:.1?}ms Max: {:.1?}ms Avg: {:.1?}ms",
            clip_filename,
            min * 1000.0,
            max * 1000.0,
            avg * 1000.0
        );

        assert!(max < MAX_MAX_TIMING_ERROR);
        assert!(avg < MAX_AVG_TIMING_ERROR);
    }
}

// Loads the haptic clip from the given file into a Player and records and returns the events
// that the Player provides.
//
// Also verifies that the events arrive roughly at the correct time.
pub fn record_events_from_player(clip_filename: &str) -> Vec<Event> {
    let clip = load_file_from_test_data(clip_filename);
    let mut recorder = PlayerEventRecorder::new();
    recorder.player().load(clip.clone()).unwrap();
    recorder.player().play().unwrap();

    // Sleep until the clip has finished playing, so that all events are collected.
    // Sleep twice as long as needed to be sure that the test still works when playback
    // takes a bit longer due to scheduling variations.
    std::thread::sleep(clip_length(&clip) * 2);

    print_timing_errors(&mut recorder, clip_filename);
    recorder.recorded_events()
}

// Compares the events a HapticEventProvider and a Player provide for a haptic clip loaded
// from the given file with a list of expected events.
pub fn compare_events(clip_filename: &str, expected_events: &[Event]) {
    let provider_events = record_events_from_provider(clip_filename);
    let player_events = record_events_from_player(clip_filename);
    assert_eq!(provider_events, expected_events);
    assert_eq!(player_events, expected_events);
}

/// Plays the clip for a bit (until expected_pre_seek_events are received), then
/// seeks to the specified time, then plays until the end of the clip.
/// Verifies that the HapticEventProvider provides the events given in
/// expected_pre_seek_events and expected_post_seek_events.
pub fn compare_seek_events_from_provider(
    clip_filename: &str,
    expected_pre_seek_events: &[Event],
    seek_time: f32,
    expected_post_seek_events: &[Event],
) {
    let clip = load_file_from_test_data(clip_filename);
    let mut provider = HapticEventProvider::new(clip);
    let actual_pre_seek_events =
        gather_events_from_provider(&mut provider, Some(expected_pre_seek_events.len()));
    assert_eq!(expected_pre_seek_events, actual_pre_seek_events);
    provider.seek(seek_time);
    let actual_post_seek_events = gather_events_from_provider(&mut provider, None);
    assert_eq!(expected_post_seek_events, actual_post_seek_events);
}

/// Similar to compare_seek_events_from_provider(), only that a streaming::Player is used instead
/// of using the HapticEventProvider directly.
///
/// When seeking past the end of a clip, the time of the event that ramps down the amplitude to 0
/// is the time of the last breakpoint, not the seek offset. Strictly speaking this is a bug, but
/// it doesn't matter as the Objective-C layer only looks at the event duration, not the event time.
/// Only the timing error checks look at the event time.
/// This bug therefore makes the timing error checks not work for cases in which we seek past the
/// end of the clip. Since fixing this properly is a bit involved, we'll just skip the timing error
/// checks for these cases. That's what skip_timing_error_check is for.
pub fn compare_seek_events_from_player(
    clip_filename: &str,
    expected_pre_seek_events: &[Event],
    seek_time: f32,
    expected_post_seek_events: &[Event],
    skip_timing_error_check: bool,
) {
    // This test is quite timing-sensitive, as it sleeps for the time specified
    // in `seek_time`, while the streaming thread is playing back events.
    // If the timing is off, then seeking will happen at the wrong time, and the
    // expected events won't match the actual events.
    if !ENABLE_TIMING_DEPENDENT_TESTS {
        return;
    }

    let clip = load_file_from_test_data(clip_filename);
    let mut recorder = PlayerEventRecorder::new();
    recorder.player().load(clip).unwrap();
    recorder.player().play().unwrap();
    if !expected_pre_seek_events.is_empty() {
        let pre_seek_duration = expected_pre_seek_events.last().unwrap().time();
        // Sleep for a little bit longer than needed to account for any delays
        std::thread::sleep(Duration::from_secs_f32(pre_seek_duration) + Duration::from_millis(5));
    }

    let actual_pre_seek_events = recorder.recorded_events();
    assert_eq!(expected_pre_seek_events, actual_pre_seek_events);
    if !skip_timing_error_check {
        print_timing_errors(&mut recorder, clip_filename);
    }
    recorder.clear_recording_data(seek_time);

    recorder.player().seek(seek_time).unwrap();
    if !expected_post_seek_events.is_empty() {
        let post_seek_duration = expected_post_seek_events.last().unwrap().time()
            - expected_post_seek_events.first().unwrap().time()
            + (-seek_time).max(0.0); // extend the duration when the seek offset is negative
        std::thread::sleep(Duration::from_secs_f32(post_seek_duration) + Duration::from_millis(5));
    }

    let actual_post_seek_events = recorder.recorded_events();
    assert_eq!(expected_post_seek_events, actual_post_seek_events);
    if !skip_timing_error_check {
        print_timing_errors(&mut recorder, clip_filename);
    }
}

pub fn compare_seek_events(
    clip_filename: &str,
    expected_pre_seek_events: &[Event],
    seek_time: f32,
    expected_post_seek_events: &[Event],
    skip_timing_error_check: bool,
) {
    compare_seek_events_from_provider(
        clip_filename,
        expected_pre_seek_events,
        seek_time,
        expected_post_seek_events,
    );
    compare_seek_events_from_player(
        clip_filename,
        expected_pre_seek_events,
        seek_time,
        expected_post_seek_events,
        skip_timing_error_check,
    );
}

pub fn init_logging() {
    let _ = Builder::from_env(Env::default().default_filter_or("trace"))
        .is_test(true)
        .try_init();
}
