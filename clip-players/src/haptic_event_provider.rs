use datamodel::v1::{AmplitudeBreakpoint, Emphasis, FrequencyBreakpoint};

/// The minimum distance, in seconds, that two breakpoints need to be spaced apart
/// in order to be considered separate breakpoints. This is used in situations
/// like deciding if an additional breakpoint is needed for ramping up to the breakpoint
/// value after a seek.
pub const MIN_BREAKPOINT_DISTANCE: f32 = 0.0001;

/// Describes a playback position in an envelope (either amplitude or frequency)
#[derive(Clone, Debug)]
enum EnvelopePosition {
    /// The position is before the initial breakpoint, and the amplitude or frequency
    /// needs to be ramped up to the value of the initial breakpoint.
    ///
    /// The initial breakpoint is the first breakpoint in the clip when starting playback
    /// at the beginning of the clip. When seeking, playback is started somewhere in the middle
    /// of the clip, and the initial breakpoint can be any breakpoint.
    BeforeInitial {
        // The events that represent the ramps to be played out.
        // When starting playback at the beginning, this is just one event, representing a ramp
        // from amplitude/frequency 0 to the value of the initial breakpoint.
        // When seeking, these are two events:
        // - One to ramp from whatever is the current amplitude/frequency to the value at the
        //   start position (which is between two breakpoints)
        // - One to ramp from the start position to the initial breakpoint
        events: Vec<Event>,

        index_of_initial_breakpoint: usize,
    },

    /// The position is at the breakpoint with the given index
    InClip { index: usize },

    /// The position is after the last breakpoint, and the amplitude needs to be
    /// ramped down to 0. This variant is only used for the amplitude envelope,
    /// as the frequency isn't ramped down after the last breakpoint.
    AfterLast,

    /// Invalid position, for when playback has stopped, ended or not started yet.
    None,
}

/// An amplitude event provided by HapticEventProvider.
///
/// The event describes a change in the amplitude from the current value to
/// `amplitude`. This change shall be interpolated over the event duration,
/// like a ramp.
///
/// The emphasis shall be played at the beginning of the event.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct AmplitudeEvent {
    /// Start time of the event, as an offset in seconds from the start of the clip
    pub time: f32,

    /// Duration of the amplitude change, in seconds
    pub duration: f32,

    /// The amplitude at the end of the change
    pub amplitude: f32,

    // Since Option is not possible for repr(C), all fields of `emphasis` are
    // f32::NAN for amplitude events without emphasis.
    pub emphasis: Emphasis,
}

/// Returns true if both values are equal or if both a NAN
fn eq_f32_no_nan(a: f32, b: f32) -> bool {
    a == b || (a.is_nan() && b.is_nan())
}

/// Custom PartialEq implementation so that two events without emphasis compare equal.
///
/// This is needed since events without emphasis use NAN as the emphasis values, which
/// don't compare equal by default.
impl PartialEq for AmplitudeEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
            && self.duration == other.duration
            && self.amplitude == other.amplitude
            && eq_f32_no_nan(self.emphasis.amplitude, other.emphasis.amplitude)
            && eq_f32_no_nan(self.emphasis.frequency, other.emphasis.frequency)
    }
}

impl AmplitudeEvent {
    fn apply_amplitude_multiplication(&mut self, multiplication_factor: f32) {
        // What we actually want to send to Core Haptics is sqrt(amplitude) * multiplication_factor.
        // However, it is interfaces/ios/LofeltHaptics/LofeltHaptics/CoreHapticsPlayer.m that
        // communicates directly with Core Haptics, and it only receives self.amplitude (as calculated
        // below) from streaming, not multiplication_factor. It square roots this value before sending
        // it to Core Haptics. So to get the net value we want we calculate:
        //     amplitude * multiplication_factor^2
        // Then CoreHapticsPlayer will do:
        //     sqrt(amplitude * multiplication_factor^2)
        // Which equals:
        //     sqrt(amplitude) * multiplication_factor
        // Which is what we wanted.
        // The same applies for self.emphasis.amplitude
        //
        // We apply multiplication_factor once to let the sign take effect (if it has a negative sign
        // it will get clamped to zero) and then a second time to account for the above.
        self.amplitude = (self.amplitude * multiplication_factor).max(0.0);
        self.amplitude = (self.amplitude * multiplication_factor).min(1.0);
        if !self.emphasis.amplitude.is_nan() {
            self.emphasis.amplitude = (self.emphasis.amplitude * multiplication_factor).max(0.0);
            self.emphasis.amplitude = (self.emphasis.amplitude * multiplication_factor).min(1.0);
        }
    }

    fn apply_frequency_shift(&mut self, shift: f32) {
        if !self.emphasis.frequency.is_nan() {
            self.emphasis.frequency = (self.emphasis.frequency + shift).min(1.0).max(0.0);
        }
    }
}

/// Same as AmplitudeEvent, but for frequency changes
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct FrequencyEvent {
    pub time: f32,
    pub duration: f32,
    pub frequency: f32,
}

impl FrequencyEvent {
    fn apply_frequency_shift(&mut self, shift: f32) {
        self.frequency = (self.frequency + shift).min(1.0).max(0.0);
    }
}

/// An event provided by the HapticEventProvider, which can either be an amplitude
/// or a frequency event.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    Amplitude(AmplitudeEvent),
    Frequency(FrequencyEvent),
}

impl Event {
    /// Creates an Event describing a ramp from `current` to `next`
    fn from_amplitude_breakpoints(
        current: &AmplitudeBreakpoint,
        next: &AmplitudeBreakpoint,
    ) -> Event {
        Event::Amplitude(AmplitudeEvent {
            time: current.time,
            duration: next.time - current.time,
            amplitude: next.amplitude,
            emphasis: Emphasis {
                amplitude: current.emphasis.map_or(f32::NAN, |e| e.amplitude),
                frequency: current.emphasis.map_or(f32::NAN, |e| e.frequency),
            },
        })
    }

    /// Creates an Event describing a ramp from `current` to `next`
    fn from_frequency_breakpoints(
        current: &FrequencyBreakpoint,
        next: &FrequencyBreakpoint,
    ) -> Event {
        Event::Frequency(FrequencyEvent {
            time: current.time,
            duration: next.time - current.time,
            frequency: next.frequency,
        })
    }

    pub fn time(&self) -> f32 {
        match self {
            Event::Frequency(event) => event.time,
            Event::Amplitude(event) => event.time,
        }
    }

    pub fn apply_amplitude_multiplication(&mut self, multiplication_factor: f32) {
        if let Event::Amplitude(amplitude_event) = self {
            amplitude_event.apply_amplitude_multiplication(multiplication_factor)
        }
    }

    pub fn apply_frequency_shift(&mut self, shift: f32) {
        match self {
            Event::Amplitude(amplitude_event) => amplitude_event.apply_frequency_shift(shift),
            Event::Frequency(frequency_event) => frequency_event.apply_frequency_shift(shift),
        }
    }

    pub fn immediate_stop_event() -> Event {
        Event::Amplitude(AmplitudeEvent {
            time: 0.0,
            duration: 0.0,
            amplitude: 0.0,
            emphasis: Emphasis {
                amplitude: f32::NAN,
                frequency: f32::NAN,
            },
        })
    }
}

/// The result of peeking the next haptic event
struct PeekedEvent {
    /// The peeked event itself, can be None if the end has been reached
    event: Option<Event>,

    /// The position inside the amplitude envelope after the operation
    new_amplitude_position: EnvelopePosition,

    /// The position inside the frequency envelope after the operation
    new_frequency_position: EnvelopePosition,
}

/// Provides haptic events from a haptic clip.
///
/// Acts like an iterator over a haptic clip, providing one haptic event in each iteration.
///
/// When starting playback with seek(), one or two haptic events per envelope
/// are provided to ramp up the amplitude and frequency to the value of the initial breakpoint.
///
/// Inside the clip, one haptic event per breakpoint is provided, describing a ramp from
/// the breakpoint to the next.
///
/// At the end of the clip, one haptic event is provided to ramp down the amplitude from
/// the last breakpoint down to zero. The frequency is not ramped down at the end, and stays
/// at the value of the last breakpoint.
pub struct HapticEventProvider {
    clip: datamodel::latest::DataModel,

    /// The current playback position inside the amplitude envelope
    amplitude_position: EnvelopePosition,

    /// The current playback position inside the frequency envelope
    frequency_position: EnvelopePosition,

    /// A multiplication factor that is applied to every amplitude event
    amplitude_multiplication: f32,

    /// A frequency shift that is applied to every frequency event and to every
    /// emphasis of an amplitude event
    frequency_shift: f32,
}

impl HapticEventProvider {
    /// Creates a new HapticEventProvider that is positioned at the beginning of the clip
    pub fn new(clip: datamodel::latest::DataModel) -> Self {
        let mut result = Self {
            clip,
            amplitude_position: EnvelopePosition::None,
            frequency_position: EnvelopePosition::None,
            amplitude_multiplication: 1.0,
            frequency_shift: 0.0,
        };
        result.seek(0.0);
        result
    }

    pub fn set_amplitude_multiplication(&mut self, multiplication_factor: f32) {
        self.amplitude_multiplication = multiplication_factor;
    }

    pub fn set_frequency_shift(&mut self, shift: f32) {
        self.frequency_shift = shift;
    }

    /// Sets the playback position to AfterLast.
    ///
    /// One last event to ramp down the amplitude will be provided. After that,
    /// No further events will be provided until calling start() or seek() again.
    pub fn stop(&mut self) {
        self.amplitude_position = EnvelopePosition::AfterLast;
        self.frequency_position = EnvelopePosition::None;
    }

    /// Returns the envelope position of the amplitude envelope after seeking
    fn amplitude_position_for_seek(&mut self, seek_time: f32) -> EnvelopePosition {
        // Find the initial breakpoint, which is the breakpoint that is first after the seek offset
        let envelope = &self.clip.signals.continuous.envelopes.amplitude;
        let index_of_initial_breakpoint = match envelope
            .binary_search_by(|breakpoint| breakpoint.time.partial_cmp(&seek_time).unwrap())
        {
            Ok(index) => index,
            Err(index) => index,
        };
        let initial_breakpoint = envelope.get(index_of_initial_breakpoint);

        match initial_breakpoint {
            // An initial breakpoint was found, create event(s) to ramp up to it
            Some(initial_breakpoint) => {
                // Find the breakpoint right before the initial breakpoint, if any
                let previous_breakpoint = if index_of_initial_breakpoint > 0 {
                    envelope.get(index_of_initial_breakpoint - 1)
                } else {
                    None
                };

                // Create the events to put into EnvelopePosition::BeforeInitial
                let events = match previous_breakpoint {
                    // If there is a previous breakpoint, create two events:
                    // 1. A ramp from whatever the current amplitude is to the amplitude
                    //    at seek offset. This ramp has a duration of 0. The amplitude at seek offset
                    //    is an interpolation between the amplitude of the previous breakpoint and
                    //    the amplitude of the initial breakpoint.
                    // 2. A ramp from the seek offset to the initial breakpoint
                    Some(previous_breakpoint) => {
                        let interpolated_breakpoint =
                            AmplitudeBreakpoint::from_interpolated_breakpoints(
                                previous_breakpoint,
                                initial_breakpoint,
                                seek_time,
                            );

                        // Ramp 1, instantly change amplitude to the amplitude
                        // at seek offset
                        let mut events = vec![Event::Amplitude(AmplitudeEvent {
                            time: seek_time,
                            duration: 0.0,
                            amplitude: interpolated_breakpoint.amplitude,
                            emphasis: Emphasis {
                                amplitude: f32::NAN,
                                frequency: f32::NAN,
                            },
                        })];

                        // Ramp 2, from seek offset to the initial breakpoint.
                        // Redundant if the interpolated breakpoint is at the same
                        // position as the initial breakpoint, i.e. if we sought to
                        // a time that already has a breakpoint.
                        if (initial_breakpoint.time - interpolated_breakpoint.time).abs()
                            > MIN_BREAKPOINT_DISTANCE
                        {
                            events.push(Event::from_amplitude_breakpoints(
                                &interpolated_breakpoint,
                                initial_breakpoint,
                            ));
                        }
                        events
                    }
                    // No previous breakpoint means we sought to a position before the
                    // first breakpoint in the clip. In that case, just create a ramp
                    // to the first breakpoint.
                    None => vec![Event::from_amplitude_breakpoints(
                        &AmplitudeBreakpoint {
                            time: seek_time,
                            amplitude: 0.0,
                            emphasis: None,
                        },
                        initial_breakpoint,
                    )],
                };
                EnvelopePosition::BeforeInitial {
                    events,
                    index_of_initial_breakpoint,
                }
            }

            // No initial breakpoint was found, probably because the seek offset is after the last
            // breakpoint
            None => {
                if !matches!(self.amplitude_position, EnvelopePosition::None) {
                    // Provide one final event to ramp down the amplitude to 0
                    EnvelopePosition::AfterLast
                } else {
                    EnvelopePosition::None
                }
            }
        }
    }

    /// Same as amplitude_position_for_seek(), but for the frequency envelope.
    ///
    /// Only the commented parts differ.
    fn frequency_position_for_seek(
        &mut self,
        seek_time: f32,
        amplitude_position: &EnvelopePosition,
    ) -> EnvelopePosition {
        let amplitude_position_at_end = matches!(
            amplitude_position,
            EnvelopePosition::AfterLast | EnvelopePosition::None
        );
        // If the position in the amplitude envelope is already at the end, no need to provide
        // frequency events as the amplitude is 0.
        if amplitude_position_at_end {
            return EnvelopePosition::None;
        }

        let envelope = match &self.clip.signals.continuous.envelopes.frequency {
            Some(envelope) => envelope,
            None => return EnvelopePosition::None,
        };

        let index_of_initial_breakpoint = match envelope
            .binary_search_by(|breakpoint| breakpoint.time.partial_cmp(&seek_time).unwrap())
        {
            Ok(index) => index,
            Err(index) => index,
        };
        let initial_breakpoint = envelope.get(index_of_initial_breakpoint);
        match initial_breakpoint {
            Some(initial_breakpoint) => {
                let previous_breakpoint = if index_of_initial_breakpoint > 0 {
                    envelope.get(index_of_initial_breakpoint - 1)
                } else {
                    None
                };
                let events = match previous_breakpoint {
                    Some(previous_breakpoint) => {
                        let interpolated_breakpoint =
                            FrequencyBreakpoint::from_interpolated_breakpoints(
                                previous_breakpoint,
                                initial_breakpoint,
                                seek_time,
                            );
                        let mut events = vec![Event::Frequency(FrequencyEvent {
                            time: seek_time,
                            duration: 0.0,
                            frequency: interpolated_breakpoint.frequency,
                        })];

                        if (initial_breakpoint.time - interpolated_breakpoint.time).abs()
                            > MIN_BREAKPOINT_DISTANCE
                        {
                            events.push(Event::from_frequency_breakpoints(
                                &interpolated_breakpoint,
                                initial_breakpoint,
                            ));
                        }
                        events
                    }
                    None => vec![Event::from_frequency_breakpoints(
                        &FrequencyBreakpoint {
                            time: seek_time,
                            frequency: 0.0,
                        },
                        initial_breakpoint,
                    )],
                };
                EnvelopePosition::BeforeInitial {
                    events,
                    index_of_initial_breakpoint,
                }
            }

            // No initial breakpoint was found, probably because the seek offset is after the last
            // breakpoint
            None => {
                if amplitude_position_at_end {
                    EnvelopePosition::None
                } else {
                    // The frequency position is at the end, but the amplitude position isn't. In such
                    // a situation, the frequency should be that of the last frequency breakpoint, so that
                    // the rest of the playback happens in that frequency. Therefore seek to the last
                    // frequency breakpoint instead, so that the frequency changes.
                    match envelope.last() {
                        Some(last_breakpoint) => {
                            let seek_time = last_breakpoint.time;
                            self.frequency_position_for_seek(seek_time, amplitude_position)
                        }
                        None => EnvelopePosition::None,
                    }
                }
            }
        }
    }

    /// Sets the playback position to the specified time after the beginning of the clip.
    pub fn seek(&mut self, seek_time: f32) {
        let seek_time = seek_time.max(0.0);
        self.amplitude_position = self.amplitude_position_for_seek(seek_time);
        self.frequency_position =
            self.frequency_position_for_seek(seek_time, &self.amplitude_position.clone());
    }

    /// Returns the start time of the next event, without advancing the position
    pub fn peek_event_start_time(&self) -> Option<f32> {
        self.peek_event(&self.amplitude_position, &self.frequency_position)
            .event
            .map(|event| event.time())
    }

    /// Returns the next event and advances the playback position
    pub fn get_next_event(&mut self) -> Option<Event> {
        let peeked_event = self.peek_event(&self.amplitude_position, &self.frequency_position);
        self.amplitude_position = peeked_event.new_amplitude_position;
        self.frequency_position = peeked_event.new_frequency_position;
        peeked_event.event
    }

    /// Returns the next event created at `position` in the amplitude envelope, together with the
    /// amplitude envelope position that follows next
    fn peek_amplitude_event(
        &self,
        position: &EnvelopePosition,
    ) -> (Option<Event>, EnvelopePosition) {
        let envelope = &self.clip.signals.continuous.envelopes.amplitude;
        match *position {
            //
            // Before the initial breakpoint: Return all events stored in EnvelopePosition::BeforeInitial
            //
            EnvelopePosition::BeforeInitial {
                ref events,
                index_of_initial_breakpoint,
            } => {
                match events.first() {
                    // EnvelopePosition::BeforeInitial has at least one event. Return the first one
                    // and continue with a new EnvelopePosition::BeforeInitial that has the first
                    // event removed.
                    Some(event) => (
                        Some(*event),
                        EnvelopePosition::BeforeInitial {
                            events: events[1..].to_vec(),
                            index_of_initial_breakpoint,
                        },
                    ),
                    // No events stored in EnvelopePosition::BeforeInitial anymore, continue
                    // with EnvelopePosition::InClip
                    None => self.peek_amplitude_event(&EnvelopePosition::InClip {
                        index: index_of_initial_breakpoint,
                    }),
                }
            }

            //
            // In the clip: Create a ramp from the current to the next breakpoint.
            //
            EnvelopePosition::InClip { index } => {
                match envelope.get(index) {
                    Some(current_breakpoint) => {
                        match envelope.get(index + 1) {
                            None => {
                                // We reached the end of the amplitude envelope. Ramp down the amplitude to 0 and finish.
                                self.peek_amplitude_event(&EnvelopePosition::AfterLast)
                            }
                            Some(next_breakpoint) => (
                                Some(Event::from_amplitude_breakpoints(
                                    current_breakpoint,
                                    next_breakpoint,
                                )),
                                EnvelopePosition::InClip { index: index + 1 },
                            ),
                        }
                    }

                    // This case shouldn't happen: Within the clip, there should always be a valid
                    // current amplitude breakpoint. If there wasn't, neither the frequency nor the
                    // amplitude position should be EnvelopePosition::InClip, as playback would have ended.
                    None => self.peek_amplitude_event(&EnvelopePosition::AfterLast),
                }
            }

            //
            // Ramping down after the last amplitude breakpoint
            //
            EnvelopePosition::AfterLast => {
                match envelope.last() {
                    Some(last_breakpoint) => (
                        Some(Event::from_amplitude_breakpoints(
                            last_breakpoint,
                            &AmplitudeBreakpoint {
                                time: last_breakpoint.time,
                                amplitude: 0.0,
                                emphasis: None,
                            },
                        )),
                        EnvelopePosition::None,
                    ),

                    // Should never get to here. The amplitude envelope is validated to not be empty,
                    // and we should not be in EnvelopePosition::AfterLast in that case as well.
                    None => self.peek_amplitude_event(&EnvelopePosition::None),
                }
            }

            //
            // No position (reached end, stopped, not started): No more events
            //
            EnvelopePosition::None => (None, EnvelopePosition::None),
        }
    }

    /// Same as peek_amplitude_event(), but for the frequency envelope.
    ///
    /// The main difference is that there is no AfterLast position for the frequency
    /// envelope.
    fn peek_frequency_event(
        &self,
        position: &EnvelopePosition,
    ) -> (Option<Event>, EnvelopePosition) {
        let envelope = &self.clip.signals.continuous.envelopes.frequency;
        match *position {
            EnvelopePosition::BeforeInitial {
                ref events,
                index_of_initial_breakpoint,
            } => match events.first() {
                Some(event) => (
                    Some(*event),
                    EnvelopePosition::BeforeInitial {
                        events: events[1..].to_owned(),
                        index_of_initial_breakpoint,
                    },
                ),
                None => self.peek_frequency_event(&EnvelopePosition::InClip {
                    index: index_of_initial_breakpoint,
                }),
            },

            EnvelopePosition::InClip { index } => match envelope {
                Some(envelope) => match envelope.get(index) {
                    Some(current_breakpoint) => match envelope.get(index + 1) {
                        Some(next_breakpoint) => (
                            Some(Event::from_frequency_breakpoints(
                                current_breakpoint,
                                next_breakpoint,
                            )),
                            EnvelopePosition::InClip { index: index + 1 },
                        ),
                        None => (None, EnvelopePosition::None),
                    },
                    None => (None, EnvelopePosition::None),
                },
                None => (None, EnvelopePosition::None),
            },

            // This case shouldn't happen: Only the amplitude position can be at AfterLast, never
            // the frequency position. When reaching the end of the frequency envelope, we keep
            // playing with the same frequency and don't ramp it down.
            EnvelopePosition::AfterLast => unreachable!(),

            EnvelopePosition::None => (None, EnvelopePosition::None),
        }
    }

    /// Returns the event at the position described by `amplitude_position` and `frequency_position`.
    ///
    /// Does not advance the playback position, instead the next playback position is returned
    /// as part of PeekedEvent.
    fn peek_event(
        &self,
        amplitude_position: &EnvelopePosition,
        frequency_position: &EnvelopePosition,
    ) -> PeekedEvent {
        let (peeked_amplitude_event, new_amplitude_position) =
            self.peek_amplitude_event(amplitude_position);
        let (peeked_frequency_event, new_frequency_position) = if matches!(
            amplitude_position,
            EnvelopePosition::None | EnvelopePosition::AfterLast
        ) {
            // Don't provide any frequency events if the amplitude position is already at the end.
            // At that point the amplitude is 0, the motor is off and providing frequency events
            // wouldn't have any effect on the motor.
            (None, EnvelopePosition::None)
        } else {
            self.peek_frequency_event(frequency_position)
        };

        let amplitude_event_to_return = PeekedEvent {
            event: peeked_amplitude_event.map(|mut event| {
                event.apply_amplitude_multiplication(self.amplitude_multiplication);
                event.apply_frequency_shift(self.frequency_shift);
                event
            }),
            new_amplitude_position,
            new_frequency_position: frequency_position.clone(),
        };
        let frequency_event_to_return = PeekedEvent {
            event: peeked_frequency_event.map(|mut event| {
                event.apply_frequency_shift(self.frequency_shift);
                event
            }),
            new_amplitude_position: amplitude_position.clone(),
            new_frequency_position,
        };

        match (peeked_amplitude_event, peeked_frequency_event) {
            (None, None) => PeekedEvent {
                event: None,
                new_amplitude_position: EnvelopePosition::None,
                new_frequency_position: EnvelopePosition::None,
            },
            (Some(_), None) => amplitude_event_to_return,
            (None, Some(_)) => frequency_event_to_return,
            (Some(amplitude_event), Some(frequency_event)) => {
                if amplitude_event.time() <= frequency_event.time() {
                    amplitude_event_to_return
                } else {
                    frequency_event_to_return
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    use utils::assert_near;

    // Tests that the HapticEventProvider provides only one event after stopping.
    #[test]
    fn peek_and_get_after_stopping() {
        test_utils::init_logging();
        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let mut provider = HapticEventProvider::new(clip);
        assert!(provider.peek_event_start_time().is_some());
        assert!(provider.get_next_event().is_some());
        provider.stop();
        assert!(provider.peek_event_start_time().is_some());
        assert!(provider.get_next_event().is_some());
        assert!(provider.peek_event_start_time().is_none());
        assert!(provider.get_next_event().is_none())
    }

    #[test]
    fn amplitude_event_amplitude_multiplication() {
        let mut amplitude_event = AmplitudeEvent {
            time: 0.0,
            duration: 1.0,
            amplitude: 1.0,
            emphasis: Emphasis {
                amplitude: 0.5,
                frequency: 0.5,
            },
        };

        amplitude_event.apply_amplitude_multiplication(0.5);
        assert_near!(amplitude_event.amplitude, 0.25, f32::EPSILON);
        assert_near!(amplitude_event.emphasis.amplitude, 0.125, f32::EPSILON);
    }

    #[test]
    fn amplitude_event_amplitude_multiplication_upper_clipping() {
        let mut amplitude_event = AmplitudeEvent {
            time: 0.0,
            duration: 1.0,
            amplitude: 1.0,
            emphasis: Emphasis {
                amplitude: 1.0,
                frequency: 0.5,
            },
        };

        amplitude_event.apply_amplitude_multiplication(2.0);
        assert_near!(amplitude_event.amplitude, 1.0, f32::EPSILON);
        assert_near!(amplitude_event.emphasis.amplitude, 1.0, f32::EPSILON);
    }

    #[test]
    fn amplitude_event_amplitude_multiplication_lower_clipping() {
        let mut amplitude_event = AmplitudeEvent {
            time: 0.0,
            duration: 1.0,
            amplitude: 1.0,
            emphasis: Emphasis {
                amplitude: 1.0,
                frequency: 0.5,
            },
        };

        amplitude_event.apply_amplitude_multiplication(-1.0);
        assert_near!(amplitude_event.amplitude, 0.0, f32::EPSILON);
        assert_near!(amplitude_event.emphasis.amplitude, 0.0, f32::EPSILON);
    }

    #[test]
    fn amplitude_event_frequency_shift() {
        let mut amplitude_event = AmplitudeEvent {
            time: 0.0,
            duration: 1.0,
            amplitude: 1.0,
            emphasis: Emphasis {
                amplitude: 1.0,
                frequency: 0.5,
            },
        };

        amplitude_event.apply_frequency_shift(0.5);
        assert_near!(amplitude_event.emphasis.frequency, 1.0, f32::EPSILON);
    }

    #[test]
    fn amplitude_event_frequency_shift_upper_clipping() {
        let mut amplitude_event = AmplitudeEvent {
            time: 0.0,
            duration: 1.0,
            amplitude: 1.0,
            emphasis: Emphasis {
                amplitude: 1.0,
                frequency: 1.0,
            },
        };

        amplitude_event.apply_frequency_shift(2.0);
        assert_near!(amplitude_event.emphasis.frequency, 1.0, f32::EPSILON);
    }

    #[test]
    fn amplitude_event_frequency_shift_lower_clipping() {
        let mut amplitude_event = AmplitudeEvent {
            time: 0.0,
            duration: 1.0,
            amplitude: 1.0,
            emphasis: Emphasis {
                amplitude: 1.0,
                frequency: 0.0,
            },
        };

        amplitude_event.apply_frequency_shift(-1.0);
        assert_near!(amplitude_event.emphasis.frequency, 0.0, f32::EPSILON);
    }

    #[test]
    fn frequency_event_frequency_shift() {
        let mut frequency_event = FrequencyEvent {
            time: 0.0,
            duration: 1.0,
            frequency: 0.5,
        };

        frequency_event.apply_frequency_shift(0.5);
        assert_near!(frequency_event.frequency, 1.0, f32::EPSILON);
    }

    #[test]
    fn frequency_event_frequency_shift_upper_clipping() {
        let mut frequency_event = FrequencyEvent {
            time: 0.0,
            duration: 1.0,
            frequency: 1.0,
        };

        frequency_event.apply_frequency_shift(1.0);
        assert_near!(frequency_event.frequency, 1.0, f32::EPSILON);
    }

    #[test]
    fn frequency_event_frequency_shift_lower_clipping() {
        let mut frequency_event = FrequencyEvent {
            time: 0.0,
            duration: 1.0,
            frequency: 0.0,
        };

        frequency_event.apply_frequency_shift(-1.0);
        assert_near!(frequency_event.frequency, 0.0, f32::EPSILON);
    }
}
