// Copyright (c) Meta Platforms, Inc. and affiliates.

//! Crate for the Lofelt SDK Core library.
//! It includes multiple functions for playback of pre-authored haptic clips, independent
//! of the platform attached.
//!
//! It is the "home" for data model, error handling enums, traits, etc.

use clip_players::PreAuthoredClipPlayback;

pub use clip_players;
pub use datamodel::VersionSupport;
pub use utils::Error;

/// Class for playing pre-authored clips
pub struct HapticsController {
    /// Player to which all functionality of playing back pre-authored clips is delegated to
    pub pre_authored_clip_player: Box<dyn PreAuthoredClipPlayback>,
    /// Duration of a loaded haptic clip
    clip_duration: f32,
}

impl HapticsController {
    pub fn new(pre_authored_clip_player: Box<dyn PreAuthoredClipPlayback>) -> HapticsController {
        HapticsController {
            pre_authored_clip_player,
            clip_duration: 0.0,
        }
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
