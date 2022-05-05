//! Crate for the Lofelt SDK Core library.
//! It includes multiple functions for playback of pre-authored haptic clips, independent
//! of the platform attached.
//!
//! It is the "home" for data model, error handling enums, traits, etc.

use {
    clip_players::PreAuthoredClipPlayback,
    crossbeam_channel::Sender,
    realtime_audio_to_haptics::{
        RealtimeAnalysisEvent, RealtimeAnalysisSettings, RealtimeAnalyzer,
    },
    std::{
        sync::Arc,
        thread::{self, JoinHandle},
    },
};

pub use clip_players;
pub use datamodel::VersionSupport;
pub use realtime_audio_to_haptics::RealtimeHapticEvent;
pub use utils::Error;

type RealtimeEventsCallback = dyn Fn(&[RealtimeHapticEvent]) + Send + Sync;

/// The data associated with a registered audio source
///
/// Audio sources are registered via HapticsController::register_audio_source().
struct RealtimeAudioSource {
    // The realtime analyzer to be used for this audio source
    analyzer: RealtimeAnalyzer,
    // The sender side of the channel used to pass updated analysis settings to the analyzer
    settings_sender: Sender<RealtimeAnalysisSettings>,
}

/// Class for playing pre-authored clips and realtime audio-to-haptics
pub struct HapticsController {
    /// Player to which all functionality of playing back pre-authored clips is delegated to
    pub pre_authored_clip_player: Box<dyn PreAuthoredClipPlayback>,
    /// Duration of a loaded haptic clip
    clip_duration: f32,
    // The realtime audio source, which must be registered via register_audio_source()
    // Currently only a single audio source is supported.
    realtime_audio_source: Option<RealtimeAudioSource>,
    // Handle for a thread that will receive events from realtime audio analysis
    realtime_events_thread: Option<JoinHandle<()>>,
    // A client callback for receiving realtime haptic events
    realtime_events_callback: Option<Arc<RealtimeEventsCallback>>,
    // The current settings used for
    realtime_analysis_settings: RealtimeAnalysisSettings,
}

impl HapticsController {
    pub fn new(pre_authored_clip_player: Box<dyn PreAuthoredClipPlayback>) -> HapticsController {
        HapticsController {
            pre_authored_clip_player,
            realtime_audio_source: None,
            realtime_events_thread: None,
            realtime_events_callback: None,
            realtime_analysis_settings: RealtimeAnalysisSettings::default(),
            clip_duration: 0.0,
        }
    }

    /// Sets the callback that will be called when new haptic events have been produced
    /// from audio analysis.
    pub fn set_audio_to_haptic_callback(
        &mut self,
        realtime_events_callback: impl Fn(&[RealtimeHapticEvent]) + Send + Sync + 'static,
    ) {
        self.realtime_events_callback = Some(Arc::new(realtime_events_callback));
    }

    /// Loads a pre-authored clip
    ///
    /// It also sets `clip_duration` based on the last amplitude envelope breakpoint time value
    pub fn load(&mut self, data: &str) -> Result<VersionSupport, Error> {
        self.pre_authored_clip_player.unload()?;
        let (version_support, haptic_data) =
            datamodel::latest_from_json(data).map_err(|string| Error::new(&string))?;

        self.clip_duration = haptic_data
            .signals
            .continuous
            .envelopes
            .amplitude
            .last()
            .map_or(0.0, |amp| amp.time);

        self.pre_authored_clip_player.load(haptic_data)?;
        Ok(version_support)
    }

    /// Plays back the pre-authored clip previously loaded with load()
    pub fn play(&mut self) -> Result<(), Error> {
        self.pre_authored_clip_player.play()
    }

    /// Stops playing back the pre-authored clip previously started with play()
    pub fn stop(&mut self) -> Result<(), Error> {
        self.pre_authored_clip_player.stop()
    }

    /// Seeks to the position specified with `time`
    pub fn seek(&mut self, time: f32) -> Result<(), Error> {
        self.pre_authored_clip_player.seek(time)
    }

    /// Sets the playback to repeat from the start at the end of the clip
    pub fn set_looping(&mut self, enabled: bool) -> Result<(), Error> {
        self.pre_authored_clip_player.set_looping(enabled)
    }

    /// Returns duration of the loaded audio clip
    pub fn get_clip_duration(&self) -> f32 {
        self.clip_duration
    }

    /// Sets the amplitude multiplication of the loaded clip
    pub fn set_amplitude_multiplication(
        &mut self,
        multiplication_factor: f32,
    ) -> Result<(), Error> {
        if multiplication_factor.is_nan()
            || multiplication_factor.is_infinite()
            || multiplication_factor < 0.0
        {
            return Err(Error::new(&format!(
                "Unable to apply amplitude multiplication factor {}, needs to be 0 or greater",
                multiplication_factor
            )));
        }

        self.pre_authored_clip_player
            .set_amplitude_multiplication(multiplication_factor)
    }

    /// Sets the frequency shift of the loaded clip
    pub fn set_frequency_shift(&mut self, shift: f32) -> Result<(), Error> {
        if shift.is_nan() || shift.is_infinite() || shift < -1.0 || shift > 1.0 {
            return Err(Error::new(&format!(
                "Unable to apply frequency shift {}, needs to be between -1 and 1",
                shift
            )));
        }

        self.pre_authored_clip_player.set_frequency_shift(shift)
    }

    /// Register an audio source for realtime haptic analysis
    ///
    /// An audio source must be registered before attempting to pass buffers into
    /// process_audio_buffer.
    ///
    #[allow(clippy::vec_init_then_push)]
    pub fn register_audio_source(&mut self, sample_rate: f32) -> Result<(), Error> {
        // If a large number of settings updates are sent at once, then blocking will occur once
        // the capacity is reached while waiting for the older settings to be applied. A larger
        // capacity reduces the risk of this blocking occurring at the expense of increased memory
        // usage. The value here is a guess at an appropriate value:
        //   - If we're looking for ways to reduce memory usage it could be brought down.
        //   - If there's blocking while trying to apply settings then it could be increased.
        let settings_channel_capacity = 128;
        let (settings_sender, settings_receiver) =
            crossbeam_channel::bounded(settings_channel_capacity);
        // The capacity here comes from trying to cover something like a worst-case scenario.
        // Buffer size: 8192
        // Update Rate: 1ms (this is unnecessarily frequent)
        // Sample Rate: 44100
        // 8192 / 44.1 == ~186 events per buffer
        // A capacity of 256 covers this extreme case with headroom to spare.
        // If memory usage becomes a concern then this could likely be brought down significantly.
        let events_channel_capacity = 256;
        let (events_sender, events_receiver) =
            crossbeam_channel::bounded::<RealtimeAnalysisEvent>(events_channel_capacity);

        if self.realtime_events_thread.is_none() {
            let haptic_event_callback = match &self.realtime_events_callback {
                Some(callback) => callback.clone(),
                None => {
                    return Err(Error::new(
                        "register_audio_source: Event callback not set up",
                    ))
                }
            };

            self.realtime_events_thread = Some(
                thread::Builder::new()
                    .name("audio_analyzer".to_string())
                    .spawn(move || {
                        // The user callback expects all haptic events for a single buffer in one go,
                        // so the haptic events are cached in this Vec and then when a
                        // BufferEnd event is received they get passed to the callback as a slice.
                        let mut events_for_callback = Vec::<RealtimeHapticEvent>::new();

                        while let Ok(event) = events_receiver.recv() {
                            match event {
                                RealtimeAnalysisEvent::HapticEvent(haptic_event) => {
                                    events_for_callback.push(haptic_event);
                                }
                                RealtimeAnalysisEvent::BufferEnd => {
                                    haptic_event_callback(events_for_callback.as_slice());
                                    events_for_callback.clear();
                                }
                                RealtimeAnalysisEvent::Quit => {
                                    break;
                                }
                            }
                        }
                    })
                    .map_err(|e| {
                        Error::new(&format!("Unable to start audio analyzer thread: {}", e))
                    })?,
            );
        }

        self.realtime_audio_source = Some(RealtimeAudioSource {
            analyzer: RealtimeAnalyzer::new(
                sample_rate,
                self.realtime_analysis_settings,
                settings_receiver,
                events_sender,
            ),
            settings_sender,
        });

        Ok(())
    }

    /// Unregisters an audio source previously registered with `register_audio_source()`.
    ///
    /// This will cause the audio analyzer thread to quit.
    ///
    /// `process_audio_buffer()` may only be called again after calling `register_audio_source()`
    /// again.
    pub fn unregister_audio_source(&mut self) -> Result<(), Error> {
        // Quit the audio analyzer thread
        if let Some(mut realtime_audio_source) = self.realtime_audio_source.take() {
            realtime_audio_source.analyzer.send_quit_event();
            if let Some(realtime_events_thread) = self.realtime_events_thread.take() {
                realtime_events_thread
                    .join()
                    .map_err(|_| Error::new("Unable to join audio analyzer thread"))?;
            }
        }

        Ok(())
    }

    /// Processes a realtime audio block for haptic analysis
    ///
    /// Currently only mono input is supported.
    ///
    /// Analyzed events will be generated by a RealtimeAnalyzer and then will be passed back to the
    /// client on a non-realtime thread via the callback set in set_callbacks().
    pub fn process_audio_buffer(&mut self, buffer: &[f32]) -> Result<(), Error> {
        match &mut self.realtime_audio_source {
            Some(source) => {
                source.analyzer.process_buffer(buffer);
                Ok(())
            }
            None => Err(Error::new(
                "process_audio_buffer: No registered audio source",
            )),
        }
    }

    /// Updates the settings used by realtime analysis
    ///
    /// This can be called at any time, settings will be applied to an active realtime analyzer at
    /// the start of processing the next audio buffer.
    pub fn set_realtime_analysis_settings(
        &mut self,
        settings: &RealtimeAnalysisSettings,
    ) -> Result<(), Error> {
        self.realtime_analysis_settings = *settings;

        if let Some(source) = &mut self.realtime_audio_source {
            // Half a second should be enough time for even the largest audio buffers to be
            // processed, so if room isn't available after this amount of time we can assume
            // something is wrong and we can return an error.
            let timeout = std::time::Duration::from_millis(500);
            if source
                .settings_sender
                .send_timeout(*settings, timeout)
                .is_err()
            {
                return Err(Error::new(
                    "set_realtime_analysis_settings: Unable to apply realtime settings",
                ));
            }
        }

        Ok(())
    }
}

impl Drop for HapticsController {
    fn drop(&mut self) {
        // This quits the audio analyzer thread if it's running
        let result = self.unregister_audio_source();
        if let Err(err) = result {
            log::error!("Unable to unregister audio source: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use clip_players::null;
    use std::path::Path;
    use utils::assert_near;

    fn load_file(path: &str) -> String {
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
    }

    fn load_test_file_valid_v1() -> String {
        load_file("../datamodel/src/test_data/valid_v1.haptic")
    }

    fn load_test_file_invalid_v1() -> String {
        load_file("../datamodel/src/test_data/invalid_version_v1.haptic")
    }

    #[test]
    /// Tests that a valid .haptic file can be played back. The clip is printed to stdout.
    fn test_play_from_valid_v1() {
        let clip = load_test_file_valid_v1();
        let mut haptics_controller = HapticsController::new(Box::new(null::Player::new().unwrap()));
        haptics_controller.load(&clip).unwrap();
        haptics_controller.play().unwrap();
    }

    #[test]
    ///Tests that the loading fails and returns an error when Lofelt Data is invalid
    fn test_load_from_invalid_v1() {
        let clip = load_test_file_invalid_v1();

        let mut haptics_controller = HapticsController::new(Box::new(null::Player::new().unwrap()));
        assert_eq!(
            haptics_controller.load(&clip).err(),
            Some(Error::new("Unsupported version"))
        );
        assert_eq!(
            haptics_controller.play().err(),
            Some(Error::new("Player play: no clip loaded"))
        );
    }

    #[test]
    ///Tests that old clips are unloaded
    fn test_unloading_on_invalid() {
        let clip = load_test_file_valid_v1();
        let invalid_clip = load_test_file_invalid_v1();

        let mut haptics_controller = HapticsController::new(Box::new(null::Player::new().unwrap()));
        haptics_controller.load(&clip).unwrap();
        haptics_controller.play().unwrap();

        assert_eq!(
            haptics_controller.load(&invalid_clip).err(),
            Some(Error::new("Unsupported version"))
        );
        assert_eq!(
            haptics_controller.play().err(),
            Some(Error::new("Player play: no clip loaded"))
        );
    }

    #[test]
    fn process_audio_receive_haptics() {
        const SAMPLE_RATE: usize = 4;
        let mut analysis_settings = RealtimeAnalysisSettings::default();

        // Configure the analysis to generate one event per second.
        analysis_settings.continuous_settings.time_between_updates = 1.0;

        // The events callback will be called from another thread,
        // so use a channel to pass the events back to the test's thread.
        let (events_sender, events_receiver) =
            crossbeam_channel::bounded::<Vec<RealtimeHapticEvent>>(128);
        let events_callback = move |events: &[RealtimeHapticEvent]| {
            events_sender
                .send(events.to_vec())
                .expect("Failed to send events in test callback");
        };

        let buffer_count = 4;

        {
            let mut controller = HapticsController::new(Box::new(null::Player::new().unwrap()));
            controller.set_audio_to_haptic_callback(events_callback);
            controller
                .register_audio_source(SAMPLE_RATE as f32)
                .expect("Failed to register audio source");
            controller
                .set_realtime_analysis_settings(&analysis_settings)
                .expect("Failed to apply analysis settings");

            for _ in 0..buffer_count {
                // Process buffers which are a second in length,
                // which should result in an event per buffer.
                controller
                    .process_audio_buffer(&[1.0f32; SAMPLE_RATE])
                    .expect("Failed to process audio buffer");
            }

            // The controller gets dropped at the scope end, which will close the events channel
            // (the sender has been moved into the callback which is owned by the controller).
        }

        let mut captured_events = Vec::new();

        // The use of a timeout here should be unnecessary,
        // but just in case something's gone haywire we don't want hanging tests.
        let timeout = std::time::Duration::from_secs(30);
        while let Ok(events) = events_receiver.recv_timeout(timeout) {
            captured_events.push(events);
        }

        // We expect a single haptic event per buffer.
        let expected_events_count = buffer_count;
        // Deterministically testing multithreaded code can be tricky, if there's a failure here
        // (particularly on CI) then it could be worth increasing the timeout.
        assert_eq!(captured_events.len(), expected_events_count);
    }

    #[test]
    /// Tests that an invalid clip as a duration of 0.0 and
    /// and a valid clip has a duration equal to the last amplitude envelope breakpoint time
    fn test_get_clip_duration() {
        let valid_clip = load_test_file_valid_v1();
        let invalid_clip = load_test_file_invalid_v1();
        let expected_duration: f32 = 9.961_361;
        let error_margin: f32 = f32::EPSILON;

        let mut haptics_controller = HapticsController::new(Box::new(null::Player::new().unwrap()));

        haptics_controller
            .load(&invalid_clip)
            .unwrap_or(VersionSupport::Full);
        assert_near!(0.0, haptics_controller.get_clip_duration(), f32::EPSILON);

        haptics_controller.load(&valid_clip).unwrap();
        assert_near!(
            expected_duration,
            haptics_controller.get_clip_duration(),
            error_margin
        );
    }

    /// Tests the validity of various numbers passed to set_amplitude_multiplication()
    #[test]
    fn test_amplitude_multiplication() {
        let clip = load_test_file_valid_v1();
        let mut haptics_controller = HapticsController::new(Box::new(null::Player::new().unwrap()));
        haptics_controller.load(&clip).unwrap();
        haptics_controller
            .set_amplitude_multiplication(-2.3)
            .unwrap_err();
        haptics_controller
            .set_amplitude_multiplication(f32::NAN)
            .unwrap_err();
        haptics_controller
            .set_amplitude_multiplication(0.5)
            .unwrap();
        haptics_controller.play().unwrap();
    }
}
