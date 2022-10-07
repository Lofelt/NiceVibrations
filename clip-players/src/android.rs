// Copyright (c) Meta Platforms, Inc. and affiliates.

use crossbeam_channel::{Receiver, Sender};
use datamodel::{
    emphasis::{emphasize, EmphasisParameters},
    interpolation::{InterpolationParameters, Interpolator},
    latest,
    v1::DataModel,
    waveform::{Waveform, WaveformConversionParameters},
};
use std::thread::{self, JoinHandle};
use utils::Error;

// As the callbacks, the tests in this file use closures that capture and modify variables.
// The callbacks need to be FnMut instead of Fn because the tests modify the captured variables.
pub struct Callbacks {
    #[allow(clippy::type_complexity)]
    pub load_clip: Box<dyn FnMut(&[i64], &[i32], bool) -> Result<(), Error> + Send>,
    pub play_clip: Box<dyn FnMut() -> Result<(), Error> + Send>,
    pub stop_clip: Box<dyn FnMut() -> Result<(), Error> + Send>,
    pub unload_clip: Box<dyn FnMut() -> Result<(), Error> + Send>,
    #[allow(clippy::type_complexity)]
    pub seek_clip: Box<dyn FnMut(&[i64], &[i32]) -> Result<(), Error> + Send>,
}

impl Callbacks {
    pub fn new(
        load: impl FnMut(&[i64], &[i32], bool) -> Result<(), Error> + Send + 'static,
        play: impl FnMut() -> Result<(), Error> + Send + 'static,
        stop: impl FnMut() -> Result<(), Error> + Send + 'static,
        unload: impl FnMut() -> Result<(), Error> + Send + 'static,
        seek: impl FnMut(&[i64], &[i32]) -> Result<(), Error> + Send + 'static,
    ) -> Callbacks {
        Callbacks {
            load_clip: Box::new(load),
            play_clip: Box::new(play),
            stop_clip: Box::new(stop),
            unload_clip: Box::new(unload),
            seek_clip: Box::new(seek),
        }
    }
}

fn convert_clip_to_waveform(clip: &DataModel) -> Waveform {
    let amplitude_breakpoints = &clip.signals.continuous.envelopes.amplitude;

    // Android amplitudes go from 0 to 255. Use amplitude 1 for the ducking_amplitude
    // here, not amplitude 0. At amplitude 0, the motor is turned off, and turning on
    // the motor afterwards takes long and screws up the timings of the waveform.
    // 1.1 is used here, not 1.0, to make sure the amplitude doesn't round down to
    // 0.
    let amplitude_breakpoints = emphasize(
        amplitude_breakpoints,
        EmphasisParameters {
            ducking_amplitude: 1.1 / 255.0,
            ..Default::default()
        },
    );

    //
    // Interpolate data
    //

    const Q_BITS: u32 = 8;
    let max_amplitude: i32 = 2_i32.pow(Q_BITS) - 1;

    /// The reason to use 25ms is to make sure we don’t add unnecessary
    /// breakpoints during the interpolation, thus avoiding to trigger the
    /// glitch bug (see Player::getPaddedEffect() in LofeltHaptics.java).
    /// Perceptually if you use less than 25ms, you can’t feel the difference
    /// on the interpolation.
    const MIN_TIME_STEP: f32 = 0.025;

    let interpolator = Interpolator::new(InterpolationParameters::new(Q_BITS, MIN_TIME_STEP));
    let amplitude_breakpoints = interpolator.process(&amplitude_breakpoints);

    //
    // Convert to Waveform and return
    //
    Waveform::from_breakpoints(
        &amplitude_breakpoints,
        WaveformConversionParameters { max_amplitude },
    )
}

fn apply_amplitude_multiplication(
    waveform: &Waveform,
    amplitude_multiplication_factor: f32,
) -> Waveform {
    if amplitude_multiplication_factor < 0.0 {
        return (*waveform).clone();
    }

    Waveform {
        timings: waveform.timings.clone(),
        amplitudes: waveform
            .amplitudes
            .iter()
            .map(|amplitude| {
                ((*amplitude as f32 * amplitude_multiplication_factor).min(255.0) as i32)
                    .min(255)
                    .max(0)
            })
            .collect(),
    }
}

/// A command sent from the player thread to the haptic thread
#[allow(clippy::large_enum_variant)]
enum PlayerCommand {
    Load(datamodel::latest::DataModel),
    Unload,
    Play,
    Stop,
    Seek { seek_time: f32 },
    SetAmplitudeMultiplication { multiplication_factor: f32 },
    Loop { enabled: bool },
    Quit,
}

/// The one function running in the haptic thread.
///
/// This is an infinite loop that waits for the next PlayerCommand to be received
/// in the crossbeam channel, then executes that command.
///
/// Most commands will trigger a matching callback to be called.
fn command_loop(mut callbacks: Callbacks, receiver: Receiver<PlayerCommand>) {
    // "Original" here means the clip and waveform right after loading them with
    // load(), before any seeking or amplitude multiplication is applied
    let mut original_clip: Option<latest::DataModel> = None;
    let mut original_waveform: Option<Waveform> = None;

    let mut amplitude_multiplication_factor: f32 = 1.0;
    let mut is_looping_enabled: bool = false;

    loop {
        match receiver.recv() {
            Ok(command) => match command {
                PlayerCommand::Quit => {
                    // Break out of the loop so that the thread is exited
                    break;
                }

                PlayerCommand::Load(data) => {
                    amplitude_multiplication_factor = 1.0;
                    is_looping_enabled = false;
                    original_clip = Some(data.clone());
                    let waveform = convert_clip_to_waveform(&data);

                    if let Err(error) = (callbacks.load_clip)(
                        &waveform.timings,
                        &waveform.amplitudes,
                        is_looping_enabled,
                    ) {
                        log::error!("Failed to load clip: {}", error);
                    }

                    original_waveform = Some(waveform);
                }

                PlayerCommand::Unload => {
                    original_clip = None;
                    original_waveform = None;

                    if let Err(error) = (callbacks.unload_clip)() {
                        log::error!("Failed to unload clip: {}", error);
                    }
                }

                PlayerCommand::Play => {
                    if let Err(error) = (callbacks.play_clip)() {
                        log::error!("Failed to play clip: {}", error);
                    }
                }

                PlayerCommand::Stop => {
                    if let Err(error) = (callbacks.stop_clip)() {
                        log::error!("Failed to stop playback: {}", error);
                    }
                }

                PlayerCommand::Seek { seek_time } => {
                    // Negative seek times are currently unsupported on Android, so clamp to zero
                    let seek_time = seek_time.max(0.0);
                    if !is_looping_enabled {
                        if let Some(clip) = &mut original_clip {
                            let mut clip_truncated = clip.clone();

                            let seek_result = match clip_truncated.truncate_before(seek_time) {
                                Ok(_) => {
                                    let waveform = convert_clip_to_waveform(&clip_truncated);
                                    let waveform = apply_amplitude_multiplication(
                                        &waveform,
                                        amplitude_multiplication_factor,
                                    );
                                    (callbacks.seek_clip)(&waveform.timings, &waveform.amplitudes)
                                }
                                Err(_) => {
                                    // A truncation error means that there are no breakpoints
                                    // after the seek offset value. In this case, we don't want
                                    // to raise an error but to play nothing.
                                    (callbacks.seek_clip)(&[], &[])
                                }
                            };

                            if let Err(error) = seek_result {
                                log::error!("Error seeking clip: {}", error);
                            }
                        }
                    }
                }

                PlayerCommand::SetAmplitudeMultiplication {
                    multiplication_factor,
                } => {
                    if let Some(original_waveform) = &original_waveform {
                        amplitude_multiplication_factor = multiplication_factor;
                        let waveform = apply_amplitude_multiplication(
                            original_waveform,
                            amplitude_multiplication_factor,
                        );

                        if let Err(error) = (callbacks.load_clip)(
                            &waveform.timings,
                            &waveform.amplitudes,
                            is_looping_enabled,
                        ) {
                            log::error!(
                                "Failed to load clip for changing amplitude multiplication: {}",
                                error
                            );
                        }
                    }
                }

                PlayerCommand::Loop { enabled } => {
                    is_looping_enabled = enabled;
                    if let Some(original_waveform) = &original_waveform {
                        if let Err(error) = (callbacks.load_clip)(
                            &original_waveform.timings,
                            &original_waveform.amplitudes,
                            is_looping_enabled,
                        ) {
                            log::error!("Failed to load clip for looping: {}", error);
                        }
                    }
                }
            },

            // This case shouldn't really happen, the Player is supposed to disconnect properly by
            // sending the Quit command
            Err(err) => {
                log::error!("Error receiving haptic player command: {}", err);
                break;
            }
        }
    }
}

/// Plays pre-authored haptic clips by invoking the callbacks provided in Player::new()
/// from a separate thread.
///
/// Internally, a dedicated thread is started, running a loop that receives `PlayerCommand`s
/// from the player over a crossbeam channel. This thread is referred to as the "haptic thread".
/// The haptic thread calls the appropriate callback that matches the command it received.
pub struct Player {
    sender: Sender<PlayerCommand>,

    /// JoinHandle of the haptic thread, used to properly join it when dropping the Player
    join_handle: Option<JoinHandle<()>>,

    clip_loaded: bool,
}

impl Drop for Player {
    /// Quit the haptic thread and wait for it to complete when dropping the player
    fn drop(&mut self) {
        match self.send_command(PlayerCommand::Quit, "Quit") {
            Ok(()) => {
                if let Some(join_handle) = self.join_handle.take() {
                    if join_handle.join().is_err() {
                        log::error!("Unable to join haptic thread.");
                    }
                }
            }
            Err(err) => log::error!("Unable to quit haptic thread: {}", err),
        }
    }
}

impl Player {
    pub fn new(callbacks: Callbacks) -> Result<Player, Error> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let join_handle = thread::Builder::new()
            .name("haptics".to_string())
            .spawn(move || command_loop(callbacks, receiver))
            .map_err(|e| Error::new(&format!("Unable to start haptic thread: {}", e)))?;

        Ok(Player {
            sender,
            join_handle: Some(join_handle),
            clip_loaded: false,
        })
    }

    fn send_command(&self, command: PlayerCommand, command_name: &str) -> Result<(), Error> {
        self.sender.send(command).map_err(|e| {
            Error::new(&format!(
                "Unable to send \"{}\" command to haptic thread: {}",
                command_name, e
            ))
        })
    }
}

impl crate::PreAuthoredClipPlayback for Player {
    fn load(&mut self, data_model: latest::DataModel) -> Result<(), Error> {
        self.send_command(PlayerCommand::Load(data_model), "Load")?;
        self.clip_loaded = true;
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

    fn unload(&mut self) -> Result<(), Error> {
        self.send_command(PlayerCommand::Unload, "Unload")?;
        self.clip_loaded = false;
        Ok(())
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

    fn set_frequency_shift(&mut self, _shift: f32) -> Result<(), Error> {
        Err(Error::new("Frequency shift is not supported on Android."))
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
    use crate::PreAuthoredClipPlayback;
    use datamodel::test_utils;
    use std::{
        path::Path,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
        time::Duration,
    };

    // All Player methods such as load, play and seek happen in a separate thread. Those methods
    // just send a command to the thread and then return immediately. Therefore we need to wait a
    // bit if we want to observe the result of the operation, like a callback being called.
    //
    // Not all places in the tests below use explicit waiting with `sleep()`, some tests wait
    // by letting the Player get out of scope and dropped, which will join the thread and wait
    // until it completes.
    const ASYNC_OPERATION_SLEEP_TIME_SECS: f32 = 0.150;

    fn load_test_file(path: &str) -> latest::DataModel {
        let clip =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        datamodel::latest_from_json(&clip).unwrap().1
    }

    fn create_dummy_callbacks() -> Callbacks {
        let load = |_timings: &[i64], _amplitudes: &[i32], _enabled: bool| Ok(());
        let play = || Ok(());
        let stop = || Ok(());
        let unload = || Ok(());
        let seek = |_timings: &[i64], _amplitudes: &[i32]| Ok(());
        Callbacks::new(load, play, stop, unload, seek)
    }

    /// Verifies that convert_clip_to_waveform() works correctly for a simple clip
    #[test]
    fn convert_valid_v1() {
        let clip = load_test_file("src/test_data/valid_v1.haptic");
        let actual_waveform = convert_clip_to_waveform(&clip);
        let expected_waveform = test_utils::create_waveform(&[
            (25, 51),
            (25, 57),
            (25, 63),
            (25, 70),
            (35, 76),
            (35, 67),
            (30, 1),
            (30, 255),
            (30, 1),
            (40, 96),
            (9661, 127),
        ]);
        assert_eq!(actual_waveform, expected_waveform);
    }

    /// Verifies that the correct timings and amplitudes are passed to the load callback
    #[test]
    fn load() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");

        let expected_waveform = convert_clip_to_waveform(&clip);
        let loaded_timings = Arc::new(Mutex::new(Vec::new()));
        let loaded_amplitudes = Arc::new(Mutex::new(Vec::new()));
        {
            let loaded_timings = loaded_timings.clone();
            let loaded_amplitudes = loaded_amplitudes.clone();
            let load = move |timings: &[i64], amplitudes: &[i32], _: bool| {
                *loaded_timings.lock().unwrap() = timings.to_vec();
                *loaded_amplitudes.lock().unwrap() = amplitudes.to_vec();
                Ok(())
            };
            let mut callbacks = create_dummy_callbacks();
            callbacks.load_clip = Box::new(load);
            let mut player = Player::new(callbacks).unwrap();
            player.load(clip).unwrap();
        }

        assert_eq!(&*loaded_timings.lock().unwrap(), &expected_waveform.timings);
        assert_eq!(
            &*loaded_amplitudes.lock().unwrap(),
            &expected_waveform.amplitudes
        );
    }

    // Verifies that the callbacks are called in the right order.
    #[test]
    fn callback_order() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");

        let sequence = Arc::new(Mutex::new(1));
        {
            let sequence_clone = sequence.clone();
            let load = move |_: &[i64], _: &[i32], _: bool| {
                assert_eq!(*sequence_clone.lock().unwrap(), 1);
                *sequence_clone.lock().unwrap() = 2;
                Ok(())
            };

            let sequence_clone = sequence.clone();
            let play = move || {
                assert_eq!(*sequence_clone.lock().unwrap(), 2);
                *sequence_clone.lock().unwrap() = 3;
                Ok(())
            };

            let sequence_clone = sequence.clone();
            let stop = move || {
                assert_eq!(*sequence_clone.lock().unwrap(), 3);
                *sequence_clone.lock().unwrap() = 4;
                Ok(())
            };

            let sequence_clone = sequence.clone();
            let seek = move |_: &[i64], _: &[i32]| {
                assert_eq!(*sequence_clone.lock().unwrap(), 4);
                *sequence_clone.lock().unwrap() = 5;
                Ok(())
            };

            let sequence_clone = sequence.clone();
            let unload = move || {
                assert_eq!(*sequence_clone.lock().unwrap(), 5);
                *sequence_clone.lock().unwrap() = 6;
                Ok(())
            };

            let mut player = Player::new(Callbacks::new(load, play, stop, unload, seek)).unwrap();
            player.load(clip).unwrap();
            player.play().unwrap();
            player.stop().unwrap();
            player.seek(0.5).unwrap();
            player.unload().unwrap();
        }

        assert_eq!(*sequence.lock().unwrap(), 6);
    }

    /// Verifies that an error in the play callback will not cause a panic
    #[test]
    fn play_fail() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");
        let play = || Err(utils::Error::new("Play failed!"));
        let mut callbacks = create_dummy_callbacks();
        callbacks.play_clip = Box::new(play);
        {
            let mut player = Player::new(callbacks).unwrap();
            player.load(clip).unwrap();
            player.play().unwrap();
        }
    }

    /// Verifies that an error in the load callback will not cause a panic
    #[test]
    fn load_fail() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");
        let load = |_: &[i64], _: &[i32], _: bool| Err(utils::Error::new("Load failed!"));
        let mut callbacks = create_dummy_callbacks();
        callbacks.load_clip = Box::new(load);
        {
            let mut player = Player::new(callbacks).unwrap();
            player.load(clip).unwrap();
        }
    }

    // Tests seek function
    #[test]
    fn seek() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");

        let seek_time_backward = 0.05;
        let mut clip_truncated_before = clip.clone();
        let expected_loaded_waveform = convert_clip_to_waveform(&clip);

        clip_truncated_before
            .truncate_before(seek_time_backward)
            .unwrap();

        let expected_sought_waveform = convert_clip_to_waveform(&clip_truncated_before);

        let loaded_timings = Arc::new(Mutex::new(Vec::new()));
        let loaded_amplitudes = Arc::new(Mutex::new(Vec::new()));
        let sought_timings = Arc::new(Mutex::new(Vec::new()));
        let sought_amplitudes = Arc::new(Mutex::new(Vec::new()));

        {
            let loaded_timings = loaded_timings.clone();
            let loaded_amplitudes = loaded_amplitudes.clone();
            let load = move |timings: &[i64], amplitudes: &[i32], _: bool| {
                *loaded_timings.lock().unwrap() = timings.to_vec();
                *loaded_amplitudes.lock().unwrap() = amplitudes.to_vec();
                Ok(())
            };

            let sought_timings = sought_timings.clone();
            let sought_amplitudes = sought_amplitudes.clone();
            let seek = move |timings: &[i64], amplitudes: &[i32]| {
                *sought_timings.lock().unwrap() = timings.to_vec();
                *sought_amplitudes.lock().unwrap() = amplitudes.to_vec();
                Ok(())
            };

            let mut callbacks = create_dummy_callbacks();
            callbacks.load_clip = Box::new(load);
            callbacks.seek_clip = Box::new(seek);
            let mut player = Player::new(callbacks).unwrap();
            player.load(clip).unwrap();
            player.seek(seek_time_backward).unwrap();
        }

        assert_eq!(
            &*loaded_timings.lock().unwrap(),
            &expected_loaded_waveform.timings
        );
        assert_eq!(
            &*loaded_amplitudes.lock().unwrap(),
            &expected_loaded_waveform.amplitudes
        );
        assert_eq!(
            &*sought_timings.lock().unwrap(),
            &expected_sought_waveform.timings
        );
        assert_eq!(
            &*sought_amplitudes.lock().unwrap(),
            &expected_sought_waveform.amplitudes
        );
    }

    // Tests calling seek after the end of the clip.
    //
    // When seeking to beyond the end of the clip, no haptics should be played. This
    // is done by passing an empty waveform to seekCallback(), which stores that
    // empty waveform as the loaded clip of the player. Any calls to playCallback()
    // that follow will not play anything.
    #[test]
    fn seek_to_after_last_amplitude_bp() {
        // The clip last amplitude breakpoint is at 9.96s
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");

        let seek_time = 10.0;
        let expected_loaded_waveform = convert_clip_to_waveform(&clip);

        // The waveform passed to the seek callback should be empty, as there is
        // nothing to play.
        let expected_sought_waveform = Waveform {
            timings: vec![],
            amplitudes: vec![],
        };

        let loaded_timings = Arc::new(Mutex::new(Vec::new()));
        let loaded_amplitudes = Arc::new(Mutex::new(Vec::new()));
        let sought_timings = Arc::new(Mutex::new(Vec::new()));
        let sought_amplitudes = Arc::new(Mutex::new(Vec::new()));

        {
            let loaded_timings = loaded_timings.clone();
            let loaded_amplitudes = loaded_amplitudes.clone();
            let load = move |timings: &[i64], amplitudes: &[i32], _: bool| {
                *loaded_timings.lock().unwrap() = timings.to_vec();
                *loaded_amplitudes.lock().unwrap() = amplitudes.to_vec();
                Ok(())
            };

            let sought_timings = sought_timings.clone();
            let sought_amplitudes = sought_amplitudes.clone();
            let seek = move |timings: &[i64], amplitudes: &[i32]| {
                *sought_timings.lock().unwrap() = timings.to_vec();
                *sought_amplitudes.lock().unwrap() = amplitudes.to_vec();
                Ok(())
            };

            let mut callbacks = create_dummy_callbacks();
            callbacks.load_clip = Box::new(load);
            callbacks.seek_clip = Box::new(seek);
            let mut player = Player::new(callbacks).unwrap();
            player.load(clip).unwrap();
            player.seek(seek_time).unwrap();
        }

        assert_eq!(
            &*loaded_timings.lock().unwrap(),
            &expected_loaded_waveform.timings
        );
        assert_eq!(
            &*loaded_amplitudes.lock().unwrap(),
            &expected_loaded_waveform.amplitudes
        );
        assert_eq!(
            &*sought_timings.lock().unwrap(),
            &expected_sought_waveform.timings
        );
        assert_eq!(
            &*sought_amplitudes.lock().unwrap(),
            &expected_sought_waveform.amplitudes
        );
    }

    // Tests seek 2 consecutive times:
    // - A forward seek in the clip
    // - Then a backwards seek, to prove that the seek is always applied to the loaded clip
    #[test]
    fn seek_forward_and_then_backward() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");

        let seek_time_forward = 5.0;
        let seek_time_backward = 0.11;

        let mut clip_truncated_before_forward = clip.clone();
        let mut clip_truncated_before_backward = clip.clone();

        clip_truncated_before_forward
            .truncate_before(seek_time_forward)
            .unwrap();

        clip_truncated_before_backward
            .truncate_before(seek_time_backward)
            .unwrap();

        let expected_sought_waveform_forward =
            convert_clip_to_waveform(&clip_truncated_before_forward);
        let expected_sought_waveform_backward =
            convert_clip_to_waveform(&clip_truncated_before_backward);

        let sought_timings = Arc::new(Mutex::new(Vec::new()));
        let sought_amplitudes = Arc::new(Mutex::new(Vec::new()));

        let sought_timings_clone = sought_timings.clone();
        let sought_amplitudes_clone = sought_amplitudes.clone();
        let seek = move |timings: &[i64], amplitudes: &[i32]| {
            *sought_timings_clone.lock().unwrap() = timings.to_vec();
            *sought_amplitudes_clone.lock().unwrap() = amplitudes.to_vec();
            Ok(())
        };

        let mut callbacks = create_dummy_callbacks();
        callbacks.seek_clip = Box::new(seek);
        let mut player = Player::new(callbacks).unwrap();
        player.load(clip).unwrap();

        // Seek first time and wait a bit for the seek to complete
        player.seek(seek_time_forward).unwrap();
        std::thread::sleep(Duration::from_secs_f32(ASYNC_OPERATION_SLEEP_TIME_SECS));

        assert_eq!(
            &*(sought_timings.lock().unwrap()).to_vec(),
            &expected_sought_waveform_forward.timings
        );
        assert_eq!(
            &*(sought_amplitudes.lock().unwrap()).to_vec(),
            &expected_sought_waveform_forward.amplitudes
        );

        // Seek second time and wait a bit for the seek to complete
        player.seek(seek_time_backward).unwrap();
        std::thread::sleep(Duration::from_secs_f32(ASYNC_OPERATION_SLEEP_TIME_SECS));

        assert_eq!(
            &*(sought_timings.lock().unwrap()).to_vec(),
            &expected_sought_waveform_backward.timings
        );
        assert_eq!(
            &*(sought_amplitudes.lock().unwrap()).to_vec(),
            &expected_sought_waveform_backward.amplitudes
        );
    }

    #[test]
    // Verifies that seek() fails and returns an error when no clip is loaded
    fn seek_without_load_fail() {
        let mut player = Player::new(create_dummy_callbacks()).unwrap();
        assert_eq!(
            player.seek(5.0).unwrap_err(),
            Error::new("Unable to seek, no clip loaded.")
        );
    }

    #[test]
    // Verifies that when seek is called for a negative value, the clip will be
    // played from the start
    fn seek_negative() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");

        let seek_time_negative = -1.0;
        let expected_loaded_waveform = convert_clip_to_waveform(&clip);
        let expected_sought_waveform = convert_clip_to_waveform(&clip);

        let loaded_timings = Arc::new(Mutex::new(Vec::new()));
        let loaded_amplitudes = Arc::new(Mutex::new(Vec::new()));
        let sought_timings = Arc::new(Mutex::new(Vec::new()));
        let sought_amplitudes = Arc::new(Mutex::new(Vec::new()));

        {
            let loaded_timings = loaded_timings.clone();
            let loaded_amplitudes = loaded_amplitudes.clone();
            let load = move |timings: &[i64], amplitudes: &[i32], _: bool| {
                *loaded_timings.lock().unwrap() = timings.to_vec();
                *loaded_amplitudes.lock().unwrap() = amplitudes.to_vec();
                Ok(())
            };

            let sought_timings = sought_timings.clone();
            let sought_amplitudes = sought_amplitudes.clone();
            let seek = move |timings: &[i64], amplitudes: &[i32]| {
                *sought_timings.lock().unwrap() = timings.to_vec();
                *sought_amplitudes.lock().unwrap() = amplitudes.to_vec();
                Ok(())
            };

            let mut callbacks = create_dummy_callbacks();
            callbacks.load_clip = Box::new(load);
            callbacks.seek_clip = Box::new(seek);
            let mut player = Player::new(callbacks).unwrap();
            player.load(clip).unwrap();
            player.seek(seek_time_negative).unwrap();
        }

        assert_eq!(
            &*loaded_timings.lock().unwrap(),
            &expected_loaded_waveform.timings
        );
        assert_eq!(
            &*loaded_amplitudes.lock().unwrap(),
            &expected_loaded_waveform.amplitudes
        );
        assert_eq!(
            &*sought_timings.lock().unwrap(),
            &expected_sought_waveform.timings
        );
        assert_eq!(
            &*sought_amplitudes.lock().unwrap(),
            &expected_sought_waveform.amplitudes
        );
    }

    #[test]
    fn amplitude_multiplication() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");

        #[rustfmt::skip]
        let original_timings =    [ 25,  25,  25,  25,  35,  35,  30,  30,  30,  40, 9661];
        #[rustfmt::skip]
        let original_amplitudes = [ 51,  57,  63,  70,  76,  67,   1, 255,   1,  96,  127];
        #[rustfmt::skip]
        let half_amplitudes =     [ 25,  28,  31,  35,  38,  33,   0, 127,   0,  48,   63];
        #[rustfmt::skip]
        let double_amplitudes =   [102, 114, 126, 140, 152, 134,   2, 255,   2, 192,  254];
        #[rustfmt::skip]
        let zero_amplitudes =     [  0,   0,   0,   0,   0,   0,   0,   0,   0,   0,    0];

        #[rustfmt::skip]
        let timings_seek =    [  8,  30,  30,  30,  40, 9661];
        #[rustfmt::skip]
        let amplitudes_seek = [120,   2, 255,   2, 192,  254];

        let loaded_timings = Arc::new(Mutex::new(Vec::new()));
        let loaded_amplitudes = Arc::new(Mutex::new(Vec::new()));
        let sought_timings = Arc::new(Mutex::new(Vec::new()));
        let sought_amplitudes = Arc::new(Mutex::new(Vec::new()));

        let loaded_timings_clone = loaded_timings.clone();
        let loaded_amplitudes_clone = loaded_amplitudes.clone();
        let sought_timings_clone = sought_timings.clone();
        let sought_amplitudes_clone = sought_amplitudes.clone();
        let load = move |timings: &[i64], amplitudes: &[i32], _: bool| {
            *loaded_timings_clone.lock().unwrap() = timings.to_vec();
            *loaded_amplitudes_clone.lock().unwrap() = amplitudes.to_vec();
            Ok(())
        };
        let seek = move |timings: &[i64], amplitudes: &[i32]| {
            *sought_timings_clone.lock().unwrap() = timings.to_vec();
            *sought_amplitudes_clone.lock().unwrap() = amplitudes.to_vec();
            Ok(())
        };

        let mut callbacks = create_dummy_callbacks();
        callbacks.load_clip = Box::new(load);
        callbacks.seek_clip = Box::new(seek);
        let mut player = Player::new(callbacks).unwrap();

        // Test: Setting the multiplication factor doesn't work before a clip is loaded
        player.set_amplitude_multiplication(0.7).unwrap_err();

        // Test: Just load the clip with the default multiplication factor of 1.0
        player.load(clip).unwrap();
        std::thread::sleep(Duration::from_secs_f32(ASYNC_OPERATION_SLEEP_TIME_SECS));
        assert_eq!(&*loaded_timings.lock().unwrap(), &original_timings);
        assert_eq!(&*loaded_amplitudes.lock().unwrap(), &original_amplitudes);

        // Test: Use a multiplication factor of 0.5
        player.set_amplitude_multiplication(0.5).unwrap();
        std::thread::sleep(Duration::from_secs_f32(ASYNC_OPERATION_SLEEP_TIME_SECS));
        assert_eq!(&*loaded_timings.lock().unwrap(), &original_timings);
        assert_eq!(&*loaded_amplitudes.lock().unwrap(), &half_amplitudes);

        // Test: Use a multiplication factor of 2.0
        player.set_amplitude_multiplication(2.0).unwrap();
        std::thread::sleep(Duration::from_secs_f32(ASYNC_OPERATION_SLEEP_TIME_SECS));
        assert_eq!(&*loaded_timings.lock().unwrap(), &original_timings);
        assert_eq!(&*loaded_amplitudes.lock().unwrap(), &double_amplitudes);

        // Test: Multiplication factor is also applied after seeking
        player.seek(0.162).unwrap();
        std::thread::sleep(Duration::from_secs_f32(ASYNC_OPERATION_SLEEP_TIME_SECS));
        assert_eq!(&*sought_timings.lock().unwrap(), &timings_seek);
        assert_eq!(&*sought_amplitudes.lock().unwrap(), &amplitudes_seek);

        // Test: Use a multiplication factor of 0.0
        player.set_amplitude_multiplication(0.0).unwrap();
        std::thread::sleep(Duration::from_secs_f32(ASYNC_OPERATION_SLEEP_TIME_SECS));
        assert_eq!(&*loaded_timings.lock().unwrap(), &original_timings);
        assert_eq!(&*loaded_amplitudes.lock().unwrap(), &zero_amplitudes);

        // Test: Setting the multiplication factor doesn't work after unloading the clip
        player.unload().unwrap();
        player.set_amplitude_multiplication(0.7).unwrap_err();
    }

    // Checks if the enabling looping sets the appropriate value when calling
    // the load callback
    #[test]
    fn enable_looping() {
        let clip = load_test_file("../core/datamodel/src/test_data/valid_v1.haptic");

        let loop_enable_set_expected = true;
        let loop_enable_set = Arc::new(AtomicBool::new(false));
        {
            let loop_enable_set = loop_enable_set.clone();
            let load = move |_: &[i64], _: &[i32], enabled: bool| {
                loop_enable_set.store(enabled, Ordering::SeqCst);
                Ok(())
            };

            let mut callbacks = create_dummy_callbacks();
            callbacks.load_clip = Box::new(load);
            let mut player = Player::new(callbacks).unwrap();

            player.load(clip).unwrap();
            player.set_looping(true).unwrap();
        }
        assert_eq!(
            loop_enable_set_expected,
            loop_enable_set.load(Ordering::SeqCst)
        );
    }
}
