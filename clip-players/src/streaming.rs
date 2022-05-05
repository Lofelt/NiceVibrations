use crate::haptic_event_provider::{Event, HapticEventProvider};
use crossbeam_channel::{self, Receiver, Sender};
use std::{
    thread,
    thread::JoinHandle,
    time::{Duration, Instant},
};
use utils::Error;

pub use crate::haptic_event_provider::{AmplitudeEvent, FrequencyEvent};

type AmplitudeEventCallback = dyn FnMut(AmplitudeEvent) + Send;
type FrequencyEventCallback = dyn FnMut(FrequencyEvent) + Send;
type InitThreadCallback = dyn FnMut() + Send;

pub struct Callbacks {
    pub amplitude_event: Box<AmplitudeEventCallback>,
    pub frequency_event: Box<FrequencyEventCallback>,
    pub init_thread: Box<InitThreadCallback>,
}

/// A command sent from the player thread to the streaming thread
#[allow(clippy::large_enum_variant)]
enum PlayerCommand {
    Load(datamodel::latest::DataModel),
    Unload,
    Play,
    Stop,
    Seek { seek_time: f32 },
    SetAmplitudeMultiplication { multiplication_factor: f32 },
    SetFrequencyShift { shift: f32 },
    Loop { enabled: bool },
    Quit,
}

/// Default sleep thread time value to avoid polling after starting the thread and after stop
const DEFAULT_TIME_TO_SLEEP: f32 = 10000.0;

/// Plays pre-authored haptic clips by streaming haptic events to the callbacks provided
/// in Player::new().
///
/// Internally, a dedicated thread is started, running a loop that receives `PlayerCommand`s
/// from the player over a crossbeam channel. This thread is referred to as the "streaming thread".
/// Once it is time to play the next haptic event, the streaming thread invokes the
/// matching callback.
pub struct Player {
    sender: Sender<PlayerCommand>,
    clip_loaded: bool,

    /// JoinHandle of the streaming thread, used to properly join it when dropping the Player
    join_handle: Option<JoinHandle<()>>,
}

/// Small helper that uses an HapticEventProvider to send events to the callbacks
struct EventSender {
    callbacks: Callbacks,

    /// None if no clip is loaded yet
    event_provider: Option<HapticEventProvider>,

    /// The time the clip started to play, or will start to play.
    ///
    /// If the clip started playing at another position than the beginning due to seeking,
    /// start_time is the point in time at which the beginning would have played.
    ///
    /// If the seek time was negative, then the start time may be in the future while playback is
    /// still pending.
    ///
    /// IOW, Instant::now() - start_time is the playback offset within the clip.
    ///
    /// Set to Some if a clip is currently playing.
    start_time: Option<Instant>,

    /// Set to Some if a delay should be applied when a play command is received.
    play_delay: Option<Duration>,

    /// If looping is enabled or not.
    /// If `true`, when sending the last event, the provider is sought to 0.0
    /// which will re-send events from the beginning of `clip`
    looping_enabled: bool,
}

impl EventSender {
    fn rewind(&mut self) {
        if let Some(event_provider) = self.event_provider.as_mut() {
            event_provider.seek(0.0);
        }
        self.start_time = None;
    }

    pub fn set_looping_enabled(&mut self, enabled: bool) {
        self.looping_enabled = enabled;
    }

    /// Sends a haptic event to ramp down the amplitude to zero to the amplitude callback,
    /// if event_provider is currently active.
    ///
    /// Note that there is no explicit stop callback to call, which means objects on the
    /// Objective-C side don't get stopped and destroyed when stopping here. This was a
    /// deliberate design decision to be able to quickly resume playing when play() is called
    /// again, without needing to re-create all the objects.
    fn stop(&mut self) {
        if let Some(event_provider) = self.event_provider.as_mut() {
            if self.start_time.is_some() {
                event_provider.stop();

                // stop() produces a ramp-down event, so send that right away
                self.send_next_event();

                self.rewind();
            }
        }
    }

    fn seek(&mut self, seek_time: f32) {
        if let Some(event_provider) = self.event_provider.as_mut() {
            event_provider.seek(seek_time);

            if event_provider.peek_event_start_time().is_some() {
                // If the clip is already playing, adjust the start_time to reflect the new position
                if let Some(start_time) = self.start_time {
                    let now = Instant::now();
                    let new_start_time = if seek_time >= 0.0 {
                        now - Duration::from_secs_f32(seek_time)
                    } else {
                        if now > start_time {
                            // Seeking to a negative time,
                            // and there is an active amplitude event,
                            // so send a ramp to zero before continuing
                            self.send_event(Event::immediate_stop_event());
                        }

                        // A negative seek time means that we're going to
                        // be starting playback *in the future*
                        now + Duration::from_secs_f32(-seek_time)
                    };

                    self.start_time = Some(new_start_time);
                } else {
                    // The clip is not yet playing
                    self.play_delay = if seek_time < 0.0 {
                        // Negative seek time, so the next Play command should apply a delay
                        Some(Duration::from_secs_f32(-seek_time))
                    } else {
                        None
                    };
                }
            } else {
                self.rewind();
                self.play_delay = if seek_time < 0.0 {
                    Some(Duration::from_secs_f32(-seek_time))
                } else {
                    None
                };
            }
        } else {
            // This case should not happen as it is caught by clip_loaded in the Player
            log::error!("Attempting to seek in clip that is not loaded.");
        }
    }

    /// Gets the next event from the HapticEventProvider and passes it to the appropriate
    /// callback.
    ///
    /// Returns the amount of seconds until the next event occurs, or DEFAULT_TIME_TO_SLEEP
    /// if there is no next event.
    ///
    /// If event_provider or the next event is None, then there is a timeout because the
    /// thread has been idle without playing for a long time. In that case do nothing
    /// and go back to sleep for a long time.
    fn send_next_event(&mut self) {
        if let Some(event_provider) = self.event_provider.as_mut() {
            if let Some(event) = event_provider.get_next_event() {
                debug_assert!(self.start_time.is_some());
                match &event {
                    Event::Frequency(event) => (self.callbacks.frequency_event)(*event),
                    Event::Amplitude(event) => (self.callbacks.amplitude_event)(*event),
                }

                if event_provider.peek_event_start_time().is_none() {
                    // No more events to send, playback finished only if looping is not enabled.
                    // Otherwise, it will continue sending events from the beginning of
                    // the clip
                    if self.looping_enabled {
                        event_provider.seek(0.0);
                        self.start_time = Some(Instant::now());
                    } else {
                        self.rewind();
                    }
                }
            }
        }
    }

    fn send_event(&mut self, event: Event) {
        match &event {
            Event::Frequency(event) => (self.callbacks.frequency_event)(*event),
            Event::Amplitude(event) => (self.callbacks.amplitude_event)(*event),
        }
    }

    /// Returns the position of the playhead, as number of seconds from the beginning of the clip.
    ///
    /// This number can be negative if seek() was called with a negative offset before.
    /// If the clip isn't playing yet, None is returned.
    fn playhead_time(&self) -> Option<f32> {
        self.start_time.map(|start_time| {
            let now = Instant::now();
            if now > start_time {
                (now - start_time).as_secs_f32()
            } else {
                -((start_time - now).as_secs_f32())
            }
        })
    }

    fn time_to_next_event(&self) -> f32 {
        if let Some(playhead_time) = self.playhead_time() {
            if let Some(event_provider) = &self.event_provider {
                if let Some(next_event_time) = event_provider.peek_event_start_time() {
                    // Since this is based on the current time, it will automatically correct
                    // for drift.
                    // playhead_time can be negative if a negative seek time has been used,
                    // then then we automatically wait for the remaining time before 0.0,
                    // plus the first event's time.
                    return (next_event_time - playhead_time).max(0.0);
                }
            }
        }
        DEFAULT_TIME_TO_SLEEP
    }
}

/// The one function running in the streaming thread.
///
/// This is an infinite loop that waits for the next PlayerCommand to be received
/// in the crossbeam channel, then executes that command.
///
/// A HapticEventProvider is used to decide what haptic event needs to be played when.
/// When it is time to play the next haptic event, the thread wakes up (via the timeout in
/// recv_timeout()) and invokes the provided callback.
fn command_loop(mut callbacks: Callbacks, receiver: Receiver<PlayerCommand>) {
    (callbacks.init_thread)();

    let mut event_sender = EventSender {
        callbacks,
        event_provider: None,
        start_time: None,
        play_delay: None,
        looping_enabled: false,
    };

    loop {
        match receiver.recv_timeout(Duration::from_secs_f32(event_sender.time_to_next_event())) {
            Ok(command) => {
                match command {
                    PlayerCommand::Quit => {
                        // Break out of the loop so that the thread is exited
                        break;
                    }
                    PlayerCommand::Load(data) => {
                        event_sender.stop();
                        event_sender.event_provider = Some(HapticEventProvider::new(data));
                    }
                    PlayerCommand::Unload => {
                        event_sender.stop();
                        event_sender.event_provider = None;
                    }
                    PlayerCommand::Play => {
                        match event_sender.event_provider.as_mut() {
                            // This case should not happen as it is caught by clip_loaded in the Player
                            None => {
                                log::error!("Attempting to play clip that is not loaded.");
                            }
                            Some(event_provider) => {
                                // Update start_time
                                if event_sender.start_time.is_none() {
                                    event_sender.start_time =
                                        match event_provider.peek_event_start_time() {
                                            Some(next_event_time) => {
                                                let now = Instant::now();
                                                let next_event =
                                                    Duration::from_secs_f32(next_event_time);
                                                let play_delay = event_sender
                                                    .play_delay
                                                    .take()
                                                    .unwrap_or_else(|| Duration::from_secs(0));
                                                Some(now - next_event + play_delay)
                                            }
                                            None => Some(Instant::now()),
                                        };
                                }
                            }
                        }
                    }
                    PlayerCommand::Stop => {
                        event_sender.stop();
                    }
                    PlayerCommand::Seek { seek_time } => {
                        event_sender.seek(seek_time);
                    }
                    PlayerCommand::SetAmplitudeMultiplication {
                        multiplication_factor,
                    } => match event_sender.event_provider.as_mut() {
                        Some(event_provider) => {
                            event_provider.set_amplitude_multiplication(multiplication_factor);

                            // If the clip is already playing, seek to the current position.
                            // While seeking to the current position sounds like a no-op at first,
                            // it is actually useful: When the current play position is between
                            // two breakpoints, not seeking would mean the amplitude multiplication
                            // is only applied at the next breakpoint, which is still some time away.
                            // That means amplitude multiplication is not instant. When seeking, an
                            // event is created right now, so the amplitude multiplication is also
                            // applied right now.
                            if let Some(playhead_time) = event_sender.playhead_time() {
                                event_sender.seek(playhead_time);
                            }
                        }

                        // This case shouldn't happen, as it is handled by Player::set_amplitude_multiplication()
                        None => {
                            log::error!("Attempting to set amplitude multiplication failed, no clip loaded.");
                        }
                    },
                    // Same as SetAmplitudeMultiplication, but for a frequency shift instead of an
                    // amplitude multiplication
                    PlayerCommand::SetFrequencyShift { shift } => {
                        match event_sender.event_provider.as_mut() {
                            Some(event_provider) => {
                                event_provider.set_frequency_shift(shift);
                                if let Some(playhead_time) = event_sender.playhead_time() {
                                    event_sender.seek(playhead_time);
                                }
                            }
                            None => {
                                log::error!(
                                    "Attempting to set frequency shift failed, no clip loaded."
                                );
                            }
                        }
                    }
                    PlayerCommand::Loop { enabled } => {
                        if event_sender.event_provider.is_none() {
                            // This case should not happen as it is caught by clip_loaded in the Player
                            log::error!("Attempting to loop a clip that is not loaded.");
                        } else {
                            event_sender.set_looping_enabled(enabled)
                        }
                    }
                }
            }
            // Since we set the timeout to be the duration until the next haptic event occurs, getting
            // a timeout error here means that it is time to stream the next haptic event.
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => event_sender.send_next_event(),

            // This case shouldn't really happen, the Player is supposed to disconnect properly by
            // sending the Quit command
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                log::error!("Sender disconnected");
                break;
            }
        }
    }
}

impl Drop for Player {
    /// Quit the streaming thread and wait for it to complete when dropping the player
    fn drop(&mut self) {
        match self.send_command(PlayerCommand::Quit, "Quit") {
            Ok(()) => {
                if let Some(join_handle) = self.join_handle.take() {
                    // Don't attempt to join the streaming thread if drop() was called from the
                    // streaming thread itself, as that would panic.
                    //
                    // This scenario can happen when the streaming thread invokes some Objective-C
                    // callbacks that retain CoreHapticsDriver, while the main thread releases
                    // LofeltHaptics and therefore CoreHapticsDriver. At the end of the callback,
                    // CoreHapticsDriver then gets released and dealloc'd from the streaming thread,
                    // triggering this scenario.
                    //
                    // Examples when this can happen is if a streaming event is playing while the
                    // main thread releases LofeltHaptics.
                    if join_handle.thread().id() != thread::current().id()
                        && join_handle.join().is_err()
                    {
                        log::error!("Unable to join streaming thread.");
                    }
                }
            }
            Err(err) => log::error!("Unable to quit streaming thread: {}", err),
        }
    }
}

impl Player {
    pub fn new(callbacks: Callbacks) -> Result<Player, Error> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let join_handle = thread::Builder::new()
            .name("haptic_streaming".to_string())
            .spawn(move || command_loop(callbacks, receiver))
            .map_err(|e| Error::new(&format!("Unable to start haptic streaming thread: {}", e)))?;

        let player = Player {
            sender,
            clip_loaded: false,
            join_handle: Some(join_handle),
        };
        Ok(player)
    }

    fn send_command(&self, command: PlayerCommand, command_name: &str) -> Result<(), Error> {
        self.sender.send(command).map_err(|e| {
            Error::new(&format!(
                "Unable to send \"{}\" command to streaming thread: {}",
                command_name, e
            ))
        })
    }
}

impl crate::PreAuthoredClipPlayback for Player {
    fn load(&mut self, data_model: datamodel::v1::DataModel) -> Result<(), Error> {
        self.send_command(PlayerCommand::Load(data_model), "Load")?;
        self.clip_loaded = true;
        Ok(())
    }

    fn unload(&mut self) -> Result<(), Error> {
        self.send_command(PlayerCommand::Unload, "Unload")?;
        self.clip_loaded = false;
        Ok(())
    }

    fn play(&mut self) -> Result<(), Error> {
        if !self.clip_loaded {
            return Err(Error::new("Unable to play, no clip loaded."));
        }
        self.send_command(PlayerCommand::Play, "Play")
    }

    fn stop(&mut self) -> Result<(), Error> {
        if self.clip_loaded {
            self.send_command(PlayerCommand::Stop, "Stop")
        } else {
            Ok(())
        }
    }

    fn seek(&mut self, seek_time: f32) -> Result<(), Error> {
        if !self.clip_loaded {
            return Err(Error::new("Unable to seek, no clip loaded."));
        }
        self.send_command(PlayerCommand::Seek { seek_time }, "Seek")
    }

    fn set_amplitude_multiplication(&mut self, multiplication_factor: f32) -> Result<(), Error> {
        if !self.clip_loaded {
            return Err(Error::new(
                "Unable to set amplitude multiplication, no clip loaded.",
            ));
        }
        self.send_command(
            PlayerCommand::SetAmplitudeMultiplication {
                multiplication_factor,
            },
            "SetAmplitudeMultiplication",
        )
    }

    fn set_frequency_shift(&mut self, shift: f32) -> Result<(), Error> {
        if !self.clip_loaded {
            return Err(Error::new("Unable to set frequency shift, no clip loaded."));
        }

        self.send_command(
            PlayerCommand::SetFrequencyShift { shift },
            "SetFrequencyShift",
        )
    }

    fn set_looping(&mut self, enabled: bool) -> Result<(), Error> {
        if !self.clip_loaded {
            return Err(Error::new("Unable to loop, no clip loaded."));
        }
        self.send_command(PlayerCommand::Loop { enabled }, "Loop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_utils::{self, amp, emp, freq, PlayerEventRecorder},
        PreAuthoredClipPlayback,
    };
    use std::time::Duration;

    // Checks an ordinary haptic clip.
    // No emphasis, and the amplitude breakpoints are at the same time as the frequency breakpoints.
    #[test]
    fn test_normal() {
        test_utils::init_logging();
        test_utils::compare_events(
            "normal.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
                amp(0.2, 0.1, 0.2),
                freq(0.2, 0.05, 0.7),
                freq(0.25, 0.05, 0.6),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Checks a clip that contains only amplitude breakpoints, and no frequency breakpoints.
    #[test]
    fn test_amplitude_only() {
        test_utils::init_logging();
        test_utils::compare_events(
            "amplitude_only.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                amp(0.1, 0.1, 0.3),
                amp(0.2, 0.1, 0.2),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Checks a clip that contains amplitude and frequency breakpoints that don't have the same times.
    #[test]
    fn test_different_times() {
        test_utils::init_logging();
        test_utils::compare_events(
            "different_times.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.15, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.15, 0.025, 0.8),
                freq(0.175, 0.175, 0.7),
                amp(0.2, 0.1, 0.2),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Checks a clip that contains more amplitude breakpoints than frequency breakpoints.
    #[test]
    fn test_more_amplitude_breakpoints() {
        test_utils::init_logging();
        test_utils::compare_events(
            "more_amplitude_bps.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.2, 0.6),
                amp(0.2, 0.1, 0.2),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Same as test_more_amplitude_breakpoints(), only that here, there are more
    // frequency than amplitude breakpoints.
    #[test]
    fn test_more_frequency_breakpoints() {
        test_utils::init_logging();
        test_utils::compare_events(
            "more_frequency_bps.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.3, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                freq(0.1, 0.1, 0.8),
                freq(0.2, 0.05, 0.7),
                freq(0.25, 0.05, 0.6),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Checks a clip whose first amplitude breakpoint doesn't start at time 0.
    // This case can not be produced by Studio Desktop.
    // Right now the behaviour is to ramp up the amplitude from 0 from the start
    // to the first breakpoint, and not to use amplitude 0 until the first breakpoint
    // is reached.
    #[test]
    fn test_first_amplitude_breakpoint_not_at_time_0() {
        test_utils::init_logging();
        #[rustfmt::skip]
        test_utils::compare_events(
            "first_amp_bp_not_time_0.haptic",
            &[
                amp(0.0, 0.3, 0.1),
                amp(0.3, 0.1, 0.2),
                amp(0.4, 0.0, 0.0)
            ],
        );
    }

    // Same as test_first_amplitude_breakpoint_not_at_time_0(), only that here, it's
    // the first frequency breakpoint that doesn't start at 0.
    #[test]
    fn test_first_frequency_breakpoint_not_at_time_0() {
        test_utils::init_logging();
        test_utils::compare_events(
            "first_freq_bp_not_time_0.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.3, 0.2),
                freq(0.0, 0.1, 0.95),
                freq(0.1, 0.2, 0.6),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Checks a clip that contains only one frequency breakpoint.
    #[test]
    fn test_one_frequency_breakpoint() {
        test_utils::init_logging();
        test_utils::compare_events(
            "one_freq_bp.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.3, 0.2),
                freq(0.0, 0.0, 0.95),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Checks a clip that contains only one amplitude breakpoint.
    #[test]
    fn test_one_amplitude_breakpoint() {
        test_utils::init_logging();
        #[rustfmt::skip]
        test_utils::compare_events(
            "one_amp_bp.haptic",
            &[
                amp(0.0, 0.3, 0.9),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Checks a clip that contains only one amplitude breakpoint, but many
    // frequency breakpoints.
    #[test]
    fn test_one_amplitude_breakpoint_multiple_frequency_breakpoints() {
        test_utils::init_logging();
        test_utils::compare_events(
            "one_amp_bp_multiple_freq_bps.haptic",
            &[
                amp(0.0, 0.3, 0.9),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.15, 0.9),
                freq(0.15, 0.15, 0.8),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Checks an ordinary haptic clip that has one amplitude breakpoints with emphasis.
    #[test]
    fn test_normal_with_1_emphasis() {
        test_utils::init_logging();
        test_utils::compare_events(
            "normal_with_1_emphasis.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                emp(0.1, 0.1, 0.3, 0.6, 0.3),
                freq(0.1, 0.1, 0.8),
                amp(0.2, 0.1, 0.2),
                freq(0.2, 0.05, 0.7),
                freq(0.25, 0.05, 0.6),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Same as test_normal_with_1_emphasis(), only that two breakpoints have emphasis.
    #[test]
    fn test_normal_with_2_emphasis() {
        test_utils::init_logging();
        test_utils::compare_events(
            "normal_with_2_emphasis.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                emp(0.1, 0.1, 0.3, 0.6, 0.3),
                freq(0.1, 0.1, 0.8),
                amp(0.2, 0.1, 0.2),
                freq(0.2, 0.05, 0.7),
                freq(0.25, 0.05, 0.6),
                emp(0.3, 0.0, 0.0, 0.95, 0.8),
            ],
        );
    }

    // Checks a clip that has emphasis on the last amplitude breakpoint.
    #[test]
    fn emphasis_at_end() {
        test_utils::init_logging();
        test_utils::compare_events(
            "emphasis_at_end.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                amp(0.1, 0.2, 0.05),
                emp(0.3, 0.0, 0.0, 0.6, 0.3),
            ],
        );
    }

    // Like emphasis_at_end(), only that the emphasis is on the first amplitude breakpoint.
    #[test]
    fn emphasis_at_start() {
        test_utils::init_logging();
        test_utils::compare_events(
            "emphasis_at_start.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                emp(0.0, 0.1, 0.2, 0.6, 0.3),
                amp(0.1, 0.2, 0.05),
                amp(0.3, 0.0, 0.0),
            ],
        );
    }

    // Tests seeking to a position before the current playback position
    #[test]
    fn seek_backward() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
            ],
            0.05,
            &[
                amp(0.05, 0.0, 0.15),
                amp(0.05, 0.05, 0.2),
                freq(0.05, 0.0, 0.925),
                freq(0.05, 0.05, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
                amp(0.2, 0.1, 0.2),
                freq(0.2, 0.05, 0.7),
                freq(0.25, 0.05, 0.6),
                amp(0.3, 0.0, 0.0),
            ],
            false,
        );
    }

    // Tests seeking to a position after the current playback position.
    // Also tests that seeking to the same position as an existing frequency
    // breakpoint will not create a duplicate event.
    #[test]
    fn seek_forward() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
            ],
            0.25,
            &[
                amp(0.25, 0.0, 0.25),
                amp(0.25, 0.05, 0.2),
                freq(0.25, 0.0, 0.7),
                freq(0.25, 0.05, 0.6),
                amp(0.3, 0.0, 0.0),
            ],
            false,
        );
    }

    // Tests seeking backward to the beginning of the clip
    #[test]
    fn seek_to_beginning() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
            ],
            0.0,
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
                amp(0.2, 0.1, 0.2),
                freq(0.2, 0.05, 0.7),
                freq(0.25, 0.05, 0.6),
                amp(0.3, 0.0, 0.0),
            ],
            false,
        );
    }

    // Tests seeking backward to a time before the first breakpoint
    #[test]
    fn seek_to_before_first_bp() {
        test_utils::init_logging();
        #[rustfmt::skip]
        test_utils::compare_seek_events(
            "first_amp_bp_not_time_0.haptic",
            &[
                amp(0.0, 0.3, 0.1),
                amp(0.3, 0.1, 0.2),
            ],
            0.2,
            &[
                amp(0.2, 0.1, 0.1),
                amp(0.3, 0.1, 0.2),
                amp(0.4, 0.0, 0.0)
            ],
            false
        );
    }

    // Tests seeking to the last breakpoint
    #[test]
    fn seek_to_last_bp() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
            ],
            0.3,
            &[amp(0.3, 0.0, 0.2), amp(0.3, 0.0, 0.0)],
            false,
        );
    }

    // Tests seeking to after the last breakpoint
    #[test]
    fn seek_to_after_last_bp() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
            ],
            0.4,
            &[amp(0.3, 0.0, 0.0)],
            true,
        );
    }

    // Tests seeking to after the end of the last breakpoint
    #[test]
    fn seek_from_start_to_after_last_bp() {
        test_utils::init_logging();
        #[rustfmt::skip]
        test_utils::compare_seek_events(
            "normal.haptic",
            &[],
            0.4,
            &[amp(0.3, 0.0, 0.0)],
            true
        );
    }

    // Tests seeking to before a breakpoint with emphasis
    #[test]
    fn seek_to_before_emphasis() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal_with_2_emphasis.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                emp(0.1, 0.1, 0.3, 0.6, 0.3),
                freq(0.1, 0.1, 0.8),
            ],
            0.25,
            &[
                amp(0.25, 0.0, 0.25),
                amp(0.25, 0.05, 0.2),
                freq(0.25, 0.0, 0.7),
                freq(0.25, 0.05, 0.6),
                emp(0.3, 0.0, 0.0, 0.95, 0.8),
            ],
            false,
        );
    }

    // Tests seeking to a breakpoint with emphasis
    #[test]
    fn seek_to_emphasis() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal_with_2_emphasis.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                emp(0.1, 0.1, 0.3, 0.6, 0.3),
                freq(0.1, 0.1, 0.8),
            ],
            0.3,
            &[amp(0.3, 0.0, 0.2), emp(0.3, 0.0, 0.0, 0.95, 0.8)],
            false,
        );
    }

    // Tests seeking to after a breakpoint with emphasis
    #[test]
    fn seek_to_after_emphasis() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal_with_2_emphasis.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
            ],
            0.15,
            &[
                amp(0.15, 0.0, 0.25),
                amp(0.15, 0.05, 0.3),
                freq(0.15, 0.0, 0.85),
                freq(0.15, 0.05, 0.8),
                amp(0.2, 0.1, 0.2),
                freq(0.2, 0.05, 0.7),
                freq(0.25, 0.05, 0.6),
                emp(0.3, 0.0, 0.0, 0.95, 0.8),
            ],
            false,
        );
    }

    // Tests seeking to a position that is after the end of the frequency envelope,
    // but still inside the amplitude envelope.
    #[test]
    fn seek_to_after_end_of_frequency_envelope() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "different_times_2.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.05, 0.9),
            ],
            0.25,
            &[
                freq(0.2, 0.0, 0.6),
                amp(0.25, 0.0, 0.25),
                amp(0.25, 0.05, 0.2),
                amp(0.3, 0.0, 0.0),
            ],
            true,
        );
    }

    // Tests seeking to a position that is after the end of the amplitude envelope,
    // but still inside the frequency envelope.
    #[test]
    fn seek_to_after_end_of_amplitude_envelope() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "different_times.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.15, 0.9),
            ],
            0.35,
            &[amp(0.3, 0.0, 0.0)],
            true,
        );
    }

    // Tests calling seek() without calling play(), which shouldn't trigger any events.
    #[test]
    fn seek_without_playing() {
        test_utils::init_logging();

        let clip = test_utils::load_file_from_test_data("normal.haptic");

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();
        recorder.player().seek(0.25).unwrap();

        std::thread::sleep(Duration::from_secs_f32(0.6));

        assert!(recorder.recorded_events().is_empty());
        recorder.clear_recording_data(0.0);
        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_secs_f32(0.6));

        let expected_post_seek_events = [
            amp(0.25, 0.0, 0.25),
            amp(0.25, 0.05, 0.2),
            freq(0.25, 0.0, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
        ];

        assert_eq!(recorder.recorded_events(), expected_post_seek_events);
        test_utils::print_timing_errors(&mut recorder, "normal.haptic");
    }

    // Tests that calling play() after playback has completely finished will restart the
    // playback from the beginning
    #[test]
    fn play_twice_fully() {
        test_utils::init_logging();

        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let expected_events = [
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();

        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_secs_f32(0.6));
        assert_eq!(recorder.recorded_events(), expected_events);
        test_utils::print_timing_errors(&mut recorder, "normal.haptic - 1");

        recorder.clear_recording_data(0.0);

        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_secs_f32(0.6));
        assert_eq!(recorder.recorded_events(), expected_events);
        test_utils::print_timing_errors(&mut recorder, "normal.haptic - 2");
    }

    // Tests that calling play() while the clip is already playing doesn't change playback.
    #[test]
    fn play_twice() {
        test_utils::init_logging();

        let clip = test_utils::load_file_from_test_data("normal.haptic");

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();

        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_secs_f32(0.1));
        recorder.player().play().unwrap();

        std::thread::sleep(Duration::from_secs_f32(0.6));

        assert_eq!(recorder.recorded_events().len(), 10);
        test_utils::print_timing_errors(&mut recorder, "normal.haptic");
    }

    // Same as seek_from_start_to_after_last_bp(), only that one full playback is done before
    // seeking.
    // The result should be the same.
    #[test]
    fn seek_after_last_bp_after_playing_once() {
        test_utils::init_logging();

        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let expected_events = [
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();

        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_secs_f32(0.6));
        assert_eq!(recorder.recorded_events(), expected_events);
        test_utils::print_timing_errors(&mut recorder, "normal.haptic");

        recorder.clear_recording_data(0.0);
        recorder.player().seek(0.4).unwrap();
        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_secs_f32(0.6));
        assert_eq!(recorder.recorded_events(), &[amp(0.3, 0.0, 0.0)]);
    }

    // Tests that a seek to a negative position delays the start of playback
    #[test]
    fn seek_negative() {
        test_utils::init_logging();
        test_utils::compare_seek_events(
            "normal.haptic",
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
            ],
            -1.0,
            &[
                amp(0.0, 0.0, 0.1),
                amp(0.0, 0.1, 0.2),
                freq(0.0, 0.0, 0.95),
                freq(0.0, 0.1, 0.9),
                amp(0.1, 0.1, 0.3),
                freq(0.1, 0.1, 0.8),
                amp(0.2, 0.1, 0.2),
                freq(0.2, 0.05, 0.7),
                freq(0.25, 0.05, 0.6),
                amp(0.3, 0.0, 0.0),
            ],
            false,
        );
    }

    // Verifies that stopping a clip works
    #[test]
    fn stop() {
        test_utils::init_logging();

        // This test relies on timing and is too flaky on the CI
        if !test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            return;
        }

        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let mut recorder = test_utils::PlayerEventRecorder::new();
        recorder.player().load(clip.clone()).unwrap();

        // Play for 150ms, which should play out 6 of the events
        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(recorder.recorded_events().len(), 6);

        // Stop the clip and wait for a bit. The only event that should be played
        // is the event that ramps down the amplitude to zero
        recorder.player().stop().unwrap();
        std::thread::sleep(test_utils::clip_length(&clip) * 2);
        assert_eq!(recorder.recorded_events().len(), 7);
        let ramp_down_event = *recorder.recorded_events().last().unwrap();
        let expected_event = amp(0.3, 0.0, 0.0);
        assert_eq!(ramp_down_event, expected_event);

        // Stop the clip again, which should be a no-op
        recorder.player().stop().unwrap();
        std::thread::sleep(test_utils::clip_length(&clip) * 2);
        assert_eq!(recorder.recorded_events().len(), 7);
    }

    // Verifies that calling stop() while no clip is loaded doesn't produce an error.
    #[test]
    fn stop_while_not_loaded() {
        test_utils::init_logging();
        let callbacks = Callbacks {
            amplitude_event: Box::new(|_| {}),
            frequency_event: Box::new(|_| {}),
            init_thread: Box::new(|| {}),
        };
        let mut player = Player::new(callbacks).unwrap();
        player.stop().unwrap();
    }

    // Verifies that calling stop() while no clip is playing doesn't produce an error.
    #[test]
    fn stop_while_not_playing() {
        test_utils::init_logging();
        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let callbacks = Callbacks {
            amplitude_event: Box::new(|_| {}),
            frequency_event: Box::new(|_| {}),
            init_thread: Box::new(|| {}),
        };
        let mut player = Player::new(callbacks).unwrap();
        player.load(clip).unwrap();
        player.stop().unwrap();
    }

    // Verifies that unloading a clip stops playback.
    // Works the same way as the stop() test.
    #[test]
    fn unload() {
        test_utils::init_logging();

        // This test relies on timing and is too flaky on the CI
        if !test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            return;
        }

        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let mut recorder = test_utils::PlayerEventRecorder::new();
        recorder.player().load(clip.clone()).unwrap();

        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(recorder.recorded_events().len(), 6);

        recorder.player().unload().unwrap();
        std::thread::sleep(test_utils::clip_length(&clip) * 2);
        assert_eq!(recorder.recorded_events().len(), 7);
        let ramp_down_event = *recorder.recorded_events().last().unwrap();
        let expected_event = amp(0.3, 0.0, 0.0);
        assert_eq!(ramp_down_event, expected_event);

        recorder.player().unload().unwrap();
        std::thread::sleep(test_utils::clip_length(&clip) * 2);
        assert_eq!(recorder.recorded_events().len(), 7);
    }

    // Verifies that loading a clip while the player is already playing will stop playback,
    // and playing will then resume from the start.
    // Works similar to the stop() and unload() tests.
    #[test]
    fn load_twice() {
        test_utils::init_logging();

        // This test relies on timing and is too flaky on the CI
        if !test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            return;
        }

        let first_clip = test_utils::load_file_from_test_data("normal.haptic");
        let mut recorder = test_utils::PlayerEventRecorder::new();
        recorder.player().load(first_clip.clone()).unwrap();

        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(recorder.recorded_events().len(), 6);

        // Loading a a new clip should trigger a stop event, and not start playing anything of the
        // new clip yet.
        let second_clip = test_utils::load_file_from_test_data("one_amp_bp.haptic");
        recorder.player().load(second_clip.clone()).unwrap();
        std::thread::sleep(test_utils::clip_length(&first_clip) * 2);
        assert_eq!(recorder.recorded_events().len(), 7);
        let ramp_down_event = *recorder.recorded_events().last().unwrap();
        let expected_event = amp(0.3, 0.0, 0.0);
        assert_eq!(ramp_down_event, expected_event);

        // Playing the new clip should trigger all its (2) events
        recorder.player().play().unwrap();
        std::thread::sleep(test_utils::clip_length(&second_clip) * 2);
        assert_eq!(recorder.recorded_events().len(), 9);
    }

    // Verifies the amplitude multiplication is applied correctly.
    // The multiplication factor is set to 2 to also test that clipping works.
    // Note that AmplitudeEvent.apply_amplitude_multiplication() squares
    // the multiplication factor initially because it gets square rooted in
    // the iOS driver later.
    #[test]
    fn test_amplitude_multiplication() {
        test_utils::init_logging();
        let clip_filename = "normal_with_1_emphasis.haptic";
        let clip = test_utils::load_file_from_test_data(clip_filename);
        let expected_events = &[
            amp(0.0, 0.0, 0.4),
            amp(0.0, 0.1, 0.8),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            emp(0.1, 0.1, 1.0, 1.0, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.8),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
        ];

        //
        // Test HapticEventProvider
        //
        let mut provider = HapticEventProvider::new(clip.clone());
        provider.set_amplitude_multiplication(2.0);
        let actual_provider_events = test_utils::gather_events_from_provider(&mut provider, None);
        assert_eq!(actual_provider_events, expected_events);

        //
        // Test Player
        //
        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip.clone()).unwrap();
        recorder.player().set_amplitude_multiplication(2.0).unwrap();
        recorder.player().play().unwrap();
        std::thread::sleep(test_utils::clip_length(&clip) * 2);
        test_utils::print_timing_errors(&mut recorder, clip_filename);
        assert_eq!(recorder.recorded_events(), expected_events);
    }

    // Same as test_amplitude_multiplication(), but for frequency shift instead of amplitude
    // multiplication.
    #[test]
    fn test_frequency_shift() {
        test_utils::init_logging();
        let clip_filename = "normal_with_1_emphasis.haptic";
        let clip = test_utils::load_file_from_test_data(clip_filename);
        let expected_events = &[
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 1.0),
            freq(0.0, 0.1, 1.0),
            emp(0.1, 0.1, 0.3, 0.6, 0.5),
            freq(0.1, 0.1, 1.0),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.9),
            freq(0.25, 0.05, 0.8),
            amp(0.3, 0.0, 0.0),
        ];

        //
        // Test HapticEventProvider
        //
        let mut provider = HapticEventProvider::new(clip.clone());
        provider.set_frequency_shift(0.2);
        let actual_provider_events = test_utils::gather_events_from_provider(&mut provider, None);
        assert_eq!(actual_provider_events, expected_events);

        //
        // Test Player
        //
        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip.clone()).unwrap();
        recorder.player().set_frequency_shift(0.2).unwrap();
        recorder.player().play().unwrap();
        std::thread::sleep(test_utils::clip_length(&clip) * 2);
        test_utils::print_timing_errors(&mut recorder, clip_filename);
        assert_eq!(recorder.recorded_events(), expected_events);
    }

    // Tests that amplitude multiplication is correctly applied to a playing clip.
    //
    // The clip is played for 0.150s, then the amplitude multiplication is changed
    // to 0.5. This results in 3 groups of events that the player emits:
    // 1. The events before the amplitude multiplication change
    //    These are the events for the breakpoints at 0.1s and earlier.
    //    These can be found in expected_events_before_multiplication_change
    // 2. New events generated because of the amplitude multiplication change
    //    These events are generated to instantly apply the amplitude multiplication,
    //    by creating a new event that ramps up the amplitude to the new value.
    //    Due to the implementation, events for a frequency change are also generated.
    //    The time of the newly generated events is at about 0.150s.
    //    These can be found in expected_events_during_multiplication_change
    // 3. The events after the amplitude multiplication change
    //    These are the events for the breakpoints at 0.2s and later.
    //    These can be found in expected_events_after_multiplication_change
    #[test]
    fn test_live_amplitude_multiplication() {
        test_utils::init_logging();
        let clip_filename = "normal.haptic";
        let clip = test_utils::load_file_from_test_data(clip_filename);
        let expected_events_before_multiplication_change = &[
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
        ];
        let expected_events_during_multiplication_change = &[
            amp(0.15, 0.0, 0.125),
            amp(0.15, 0.05, 0.15),
            freq(0.15, 0.0, 0.85),
            freq(0.15, 0.05, 0.8),
        ];
        let expected_events_after_multiplication_change = &[
            amp(0.2, 0.1, 0.05),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();

        //
        // Events part 1
        //
        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_millis(150));
        test_utils::print_timing_errors(&mut recorder, clip_filename);
        assert_eq!(
            recorder.recorded_events(),
            expected_events_before_multiplication_change
        );

        //
        // Events part 2 and 3
        //
        recorder.clear_recording_data(0.150);
        recorder.player().set_amplitude_multiplication(0.5).unwrap();
        std::thread::sleep(Duration::from_millis(300));

        let recorded_events = recorder.recorded_events();
        let (actual_events_during_multiplication_change, actual_events_after_multiplication_change) =
            recorded_events.split_at(4);

        // Even after rounding the events to two decimal places, the sleep of 150ms
        // has a deviation that is too high for the CI
        if test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            test_utils::print_timing_errors(&mut recorder, clip_filename);
            assert_eq!(
                test_utils::rounded_events(actual_events_during_multiplication_change, 2),
                test_utils::rounded_events(expected_events_during_multiplication_change, 2)
            );
        }
        assert_eq!(
            actual_events_after_multiplication_change,
            expected_events_after_multiplication_change
        );
    }

    // Same as test_live_amplitude_multiplication(), but for frequency shift instead of
    // amplitude multiplication.
    #[test]
    fn test_live_frequency_shift() {
        test_utils::init_logging();
        let clip_filename = "normal.haptic";
        let clip = test_utils::load_file_from_test_data(clip_filename);
        let expected_events_before_frequency_shift = &[
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
        ];
        let expected_events_during_frequency_shift = &[
            amp(0.15, 0.0, 0.25),
            amp(0.15, 0.05, 0.3),
            freq(0.15, 0.0, 0.95),
            freq(0.15, 0.05, 0.9),
        ];
        let expected_events_after_frequency_shift = &[
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.8),
            freq(0.25, 0.05, 0.7),
            amp(0.3, 0.0, 0.0),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();

        //
        // Events part 1
        //
        recorder.player().play().unwrap();
        std::thread::sleep(Duration::from_millis(150));
        test_utils::print_timing_errors(&mut recorder, clip_filename);
        assert_eq!(
            recorder.recorded_events(),
            expected_events_before_frequency_shift
        );

        //
        // Events part 2 and 3
        //
        recorder.clear_recording_data(0.150);
        recorder.player().set_frequency_shift(0.1).unwrap();
        std::thread::sleep(Duration::from_millis(300));

        let recorded_events = recorder.recorded_events();
        let (actual_events_during_frequency_shift, actual_events_after_frequency_shift) =
            recorded_events.split_at(4);

        if test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            test_utils::print_timing_errors(&mut recorder, clip_filename);
            assert_eq!(
                test_utils::rounded_events(actual_events_during_frequency_shift, 2),
                test_utils::rounded_events(expected_events_during_frequency_shift, 2)
            );
        }
        assert_eq!(
            actual_events_after_frequency_shift,
            expected_events_after_frequency_shift
        );
    }

    // Verifies that enabling looping makes the playback repeat from the beginning when
    // the player reaches end
    // Should repeat at least 2 times
    #[test]
    fn loop_clip() {
        // This test relies on timing and is too flaky on the CI
        if !test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            return;
        }

        test_utils::init_logging();

        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let expected_events = [
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
            //repeat
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
            //repeat
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();

        recorder.player().set_looping(true).unwrap();

        recorder.player().play().unwrap();
        // waiting time to loop 3 times
        //      - the clip duration is 0.3s so the total time for 3 loops would
        //        be 0.9 seconds.
        //      - due to the time sensitiveness, the last event at time 0.3 with
        //        0.001 duration might not be sent so we only wait 0.89s to
        //        make sure we are not counting on that event.
        std::thread::sleep(Duration::from_secs_f32(0.89));

        assert_eq!(recorder.recorded_events(), expected_events);
    }

    // Verifies that enabling looping while the clip is playing makes the playback repeat from
    // the beginning when the player reaches end.
    // Should repeat at least 2 times.
    #[test]
    fn loop_while_playing() {
        // This test relies on timing and is too flaky on the CI
        if !test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            return;
        }

        test_utils::init_logging();

        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let expected_events = [
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
            //repeat
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
            //repeat
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();
        recorder.player().play().unwrap();

        // wait 0.2 seconds and then set looping before clip playback ends
        std::thread::sleep(Duration::from_secs_f32(0.2));
        recorder.player().set_looping(true).unwrap();

        // waiting time to loop 3 times
        //      - the clip duration is 0.3s so the total time for 3 loops would
        //        be 0.9 seconds.
        //      - since we waited 0.2 seconds to send the loop command, here we
        //        only need to wait for 0.9-0.2 = 0.7s
        //      - due to the time sensitiveness, the last event at time 0.3 with
        //        0.001 duration might not be sent so we only wait 0.69s to
        //        make sure we are not counting on that event.
        std::thread::sleep(Duration::from_secs_f32(0.69));

        assert_eq!(recorder.recorded_events(), expected_events);
    }

    #[test]
    // Verifies that disabling looping while playing stops the clip from repeating when it reaches
    // the end.
    fn looping_disabled_while_playing() {
        // This test relies on timing and is too flaky on the CI
        if !test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            return;
        }

        test_utils::init_logging();

        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let expected_events = [
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();
        recorder.player().set_looping(true).unwrap();
        recorder.player().play().unwrap();

        // wait 0.1s and then disable looping before clip playback ends
        std::thread::sleep(Duration::from_secs_f32(0.1));
        recorder.player().set_looping(false).unwrap();

        // wait 0.3s (0.1s more than the total duration) to make sure we record all events
        std::thread::sleep(Duration::from_secs_f32(0.3));

        assert_eq!(recorder.recorded_events(), expected_events);
    }

    // Verifies that seeking with looping enabled will
    // 1. Jump playback to the seek position
    // 2. Play from seek position until the end
    // 3. Loop from the beginning until the end of the clip
    #[test]
    fn loop_before_seek() {
        // This test relies on timing and is too flaky on the CI
        if !test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            return;
        }

        test_utils::init_logging();
        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let post_seek_expected_events = [
            amp(0.25, 0.0, 0.25),
            amp(0.25, 0.05, 0.2),
            freq(0.25, 0.0, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
            //repeat
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
            //repeat
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();
        recorder.player().set_looping(true).unwrap();
        recorder.player().play().unwrap();
        // wait 0.1 seconds to call seek
        std::thread::sleep(Duration::from_secs_f32(0.1));

        // clear previous events to only compare post seek events
        recorder.clear_recording_data(0.25);
        recorder.player().seek(0.25).unwrap();

        // repeating 3 times with seek would be 0.65 seconds in total:
        //      - the first loop plays for 0.1s and then is sought to 0.25.
        //        the total time played from the first loop is 0.15s
        //      - the second and third loops play for 0.3s. In total they play
        //        for 0.3+0.3+0.15 = 0.75seconds.
        //      - after calling seek, we only need to sleep for 0.65 seconds,
        //        since we already slept for 0.1 seconds before seek.
        //      - due to the time sensitiveness, the last event at time 0.3 with
        //        0.001 duration might not be sent so we only wait 0.64s to
        //        make sure we are not counting on that event.
        std::thread::sleep(Duration::from_secs_f32(0.64));

        assert_eq!(recorder.recorded_events(), post_seek_expected_events);
    }

    // Verifies that seeking past the end with looping enabled will jump immediately
    // to the beginning of the clip and repeat when it reaches the end
    #[test]
    fn loop_after_seek_past_end_of_clip() {
        // This test relies on timing and is too flaky on the CI
        if !test_utils::ENABLE_TIMING_DEPENDENT_TESTS {
            return;
        }

        test_utils::init_logging();

        let clip = test_utils::load_file_from_test_data("normal.haptic");
        let post_seek_expected_events = [
            amp(0.3, 0.0, 0.0),
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
            //repeat
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
            amp(0.3, 0.0, 0.0),
            //repeat
            amp(0.0, 0.0, 0.1),
            amp(0.0, 0.1, 0.2),
            freq(0.0, 0.0, 0.95),
            freq(0.0, 0.1, 0.9),
            amp(0.1, 0.1, 0.3),
            freq(0.1, 0.1, 0.8),
            amp(0.2, 0.1, 0.2),
            freq(0.2, 0.05, 0.7),
            freq(0.25, 0.05, 0.6),
        ];

        let mut recorder = PlayerEventRecorder::new();
        recorder.player().load(clip).unwrap();
        recorder.player().set_looping(true).unwrap();
        recorder.player().play().unwrap();
        // wait 0.2 seconds to call seek
        std::thread::sleep(Duration::from_secs_f32(0.2));

        // clear previous events to only compare post seek events
        recorder.clear_recording_data(0.0);

        recorder.player().seek(10.0).unwrap();

        // waiting time to loop 3 times
        //      - after calling seek, the playback will start from the beginning
        //        of the clip
        //      - the clip duration is 0.3s so the total time for 3 loops would
        //        be 0.9 seconds.
        //      - due to the time sensitiveness, the last event at time 0.3 with
        //        0.001 duration might not be sent so we only wait 0.89s to
        //        make sure we are not counting on that event.
        std::thread::sleep(Duration::from_secs_f32(0.89));

        assert_eq!(recorder.recorded_events(), post_seek_expected_events);
    }
}
