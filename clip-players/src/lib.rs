// Copyright (c) Meta Platforms, Inc. and affiliates.

pub mod android;
pub mod null;
pub mod streaming;

pub mod haptic_event_provider;

#[cfg(test)]
mod test_utils;

use datamodel::latest;
use utils::Error;

/// Plays back a pre-authored haptic clip.
pub trait PreAuthoredClipPlayback {
    /// Loads the clip and prepares it for playback.
    fn load(&mut self, data_model: latest::DataModel) -> Result<(), Error>;

    /// Unloads the clip, freeing any memory or resources taken in load().
    fn unload(&mut self) -> Result<(), Error>;

    /// Plays the clip.
    ///
    /// For optimal audio/haptic sync, the implementation of this function should
    /// be quick. Any expensive work should happen in load().
    ///
    /// play() has no effect if the clip is already playing.
    fn play(&mut self) -> Result<(), Error>;

    /// Seeks to the given position, which is specified in seconds since the beginning
    /// of the clip.
    ///
    /// For iOS, the playback state (playing or stopped) will not be changed unless seeking
    /// beyond the end of the clip.
    /// However, in Android, due to limitations in the Vibrator API, the seek callback will
    /// force-stop the playback. Nevertheless, the core is not aware of this at the moment since
    /// there's no playback state mechanism.
    ///
    /// Seeking beyond the end of the clip will stop playback. However, on iOS, if looping is
    /// enabled, seeking past the end of the clip will make playback start from the beginning of
    /// the clip.
    ///
    /// Clips are always defined to have a start time of 0, so negative seek times will result in a
    /// delay before playback starts.
    ///
    /// On iOS, if looping is enabled, playback will start from the sought position until the end
    /// of the clip, and then repeat from the beginning of the clip. However, in Android, seek will
    /// have no effect and the playback after calling `set_looping()` will always start from the
    /// beginning of the clip.
    fn seek(&mut self, seek_offset: f32) -> Result<(), Error>;

    /// Sets the playback to repeat from the beginning at the end of the clip.
    ///
    /// On Android, the changes will only be applied when `play()` is called. If `seek()` is called,
    /// it will have no effect when looping is enabled. It will always loop from start to end
    fn set_looping(&mut self, enabled: bool) -> Result<(), Error>;

    /// Stops a clip that is playing
    ///
    /// `stop()` has no effect if a clip is not playing
    fn stop(&mut self) -> Result<(), Error>;

    /// Multiplies the amplitude of every breakpoint of the clip with the given multiplication
    /// factor before playing it.
    ///
    /// A clip needs to be loaded for this method to take effect. Unloading a clip resets the
    /// multiplication factor to the default of 1.0.
    ///
    /// The multiplication factor needs to be 0 or greater.
    ///
    /// If the resulting amplitude of a breakpoint is greater than 1.0, it is clipped to 1.0. The
    /// amplitude is clipped hard, no limiter is used.
    fn set_amplitude_multiplication(&mut self, multiplication_factor: f32) -> Result<(), Error>;

    /// Adds the given shift to the frequency of every frequency breakpoint and to the frequency
    /// of every emphasis before playing the breakpoint.
    ///
    /// A clip needs to be loaded for this method to take effect. Unloading a clip resets the
    /// shift to the default of 0.0.
    ///
    /// The shift needs to be between -1.0 and 1.0.
    ///
    /// If the resulting frequency of a breakpoint is smaller than 0.0 or larger than 1.0, it is
    /// clipped to the valid range. The frequency is clipped hard, no limiter is used.
    fn set_frequency_shift(&mut self, shift: f32) -> Result<(), Error>;
}

#[cfg(test)]
mod tests {
    use crate::{null::Player, PreAuthoredClipPlayback};
    use std::path::Path;

    fn load_test_file(path: &str) -> String {
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
    }

    fn load_test_file_valid_v1() -> String {
        load_test_file("../core/datamodel/src/test_data/valid_v1.haptic")
    }

    #[test]
    fn test_platform_play() {
        let mut player = Player::new().unwrap();

        let data = load_test_file_valid_v1();
        let data_model = datamodel::latest_from_json(&data).unwrap().1;

        player.load(data_model).unwrap();
        player.play().unwrap();
        player.stop().unwrap();
    }

    #[test]
    fn test_platform_play_fail() {
        let mut player = Player::new().unwrap();
        assert!(player.play().is_err());
        assert!(player.stop().is_err());
    }
}
