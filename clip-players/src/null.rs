use datamodel::latest;
use utils::Error;

pub struct Player {
    haptic_clip: Option<latest::DataModel>,
}

impl Player {
    pub fn new() -> Result<Player, Error> {
        Ok(Player { haptic_clip: None })
    }
}

impl crate::PreAuthoredClipPlayback for Player {
    fn load(&mut self, data_model: latest::DataModel) -> Result<(), Error> {
        self.haptic_clip = Some(data_model);
        Ok(())
    }

    fn play(&mut self) -> Result<(), Error> {
        match &self.haptic_clip {
            Some(_) => Ok(()),
            None => Err(Error::new("Player play: no clip loaded")),
        }
    }

    fn stop(&mut self) -> Result<(), Error> {
        match &self.haptic_clip {
            Some(_) => Ok(()),
            None => Err(Error::new("Player stop: no clip loaded")),
        }
    }

    fn unload(&mut self) -> Result<(), Error> {
        self.haptic_clip = None;
        Ok(())
    }

    fn seek(&mut self, _seek_time: f32) -> Result<(), Error> {
        match &self.haptic_clip {
            Some(_) => Ok(()),
            None => Err(Error::new("Player seek: no clip loaded")),
        }
    }

    fn set_amplitude_multiplication(&mut self, _multiplication_factor: f32) -> Result<(), Error> {
        match &self.haptic_clip {
            Some(_) => Ok(()),
            None => Err(Error::new(
                "Player set_amplitude_multiplication: no clip loaded",
            )),
        }
    }

    fn set_frequency_shift(&mut self, _shift: f32) -> Result<(), Error> {
        match &self.haptic_clip {
            Some(_) => Ok(()),
            None => Err(Error::new("Player set_frequency_shift: no clip loaded")),
        }
    }

    fn set_looping(&mut self, _enabled: bool) -> Result<(), Error> {
        match &self.haptic_clip {
            Some(_) => {}
            None => return Err(Error::new("Player set_looping: no clip loaded")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PreAuthoredClipPlayback;
    use std::path::Path;

    fn load_test_file(path: &str) -> String {
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
    }

    fn load_test_file_valid_v1() -> String {
        load_test_file("../core/datamodel/src/test_data/valid_v1.haptic")
    }

    #[test]
    fn test_null_player_load_play_stop() {
        let mut player = Player::new().unwrap();
        let data = load_test_file_valid_v1();
        let data_model = datamodel::latest_from_json(&data).unwrap().1;

        player.load(data_model).unwrap();
        player.play().unwrap();
        player.stop().unwrap();
    }

    #[test]
    fn test_null_player_fail() {
        let mut player = Player::new().unwrap();
        assert!(player.play().is_err());
        assert!(player.stop().is_err());
    }
}
