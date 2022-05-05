//! Lofelt Data Model Version 0.2.0 - previously exported by Studio Desktop
use crate::version::{DataModelVersion, Version};
use crate::Validation;
use crate::MAX_ENVELOPE_AMPLITUDE;
use crate::MIN_ENVELOPE_AMPLITUDE;
use serde::{Deserialize, Serialize};

impl DataModelVersion for DataModel {
    const CURRENT: Version = Version {
        major: 0,
        minor: 2,
        patch: 0,
    };

    fn version(&self) -> &Version {
        &Self::CURRENT
    }
}

/// Main structure containing V0.2.0 of Lofelt Data Model
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataModel {
    #[serde(default)]
    pub version: Version,
    #[serde(default)]
    pub metadata: MetaData,
    pub voices: Voices,
}

impl Default for DataModel {
    fn default() -> Self {
        Self {
            version: Self::CURRENT,
            metadata: Default::default(),
            voices: Default::default(),
        }
    }
}

/// Metadata structure
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct MetaData {
    #[serde(default)]
    pub editor: String,
    #[serde(default)]
    pub duration: f32,
}

/// Voices data structure holding vectors of Envelopes and Transients.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct Voices {
    pub envelopes: Vec<Envelope>,
    pub transients: Vec<Envelope>,
}

pub type Envelope = Vec<Breakpoint>;

/// Breakpoint data structure representing Envelope curves or Transients.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Breakpoint {
    pub time: f32,
    pub amplitude: f32,
}

/// Validation trait implementation
/// An invalid Data Model would be one that:
/// - Has no breakpoints at all.
/// - Has no amplitude breakpoints.
/// - Breakpoint and transient amplitude values are < 0.0 or > 1.0.
/// - The breakpoint or transient time values are not consecutive.
/// - The transient amplitude and frequency values have no matching pairs or have different lengths.
impl Validation for DataModel {
    fn validate(self) -> Result<Self, String> {
        if self.voices.envelopes.is_empty() {
            return Err(String::from("V0 Validation Error: Envelopes are empty"));
        }

        if self.voices.envelopes[0].is_empty() {
            return Err(String::from(
                "V0 Validation Error: Amplitude envelope is empty",
            ));
        }

        let mut last_time: f32; // variable to keep track of the previous breakpoint time

        for envelope in self.voices.envelopes.iter() {
            last_time = 0.0;
            for breakpoint in envelope.iter() {
                if breakpoint.time.is_nan() {
                    return Err(
                        "V0 Validation Error: Timestamp of amplitude breakpoint is NaN".into(),
                    );
                }

                if breakpoint.amplitude > MAX_ENVELOPE_AMPLITUDE
                    || breakpoint.amplitude < MIN_ENVELOPE_AMPLITUDE
                {
                    return Err(format!(
                        "V0 Validation Error: Breakpoint amplitude out of range: {}",
                        breakpoint.amplitude,
                    ));
                }

                if last_time > breakpoint.time {
                    return Err(format!(
                        "V0 Validation Error: Breakpoint times not consecutive: {} after {}",
                        breakpoint.time, last_time,
                    ));
                }

                last_time = breakpoint.time;
            }

            if last_time > self.metadata.duration {
                return Err(format!(
                    "V0 Validation Error: event time: {} is greater than the file duration: {}",
                    last_time, self.metadata.duration
                ));
            }
        }

        if !self.voices.transients.is_empty() {
            if self.voices.transients.len() != 2 {
                return Err(String::from(
                    "V0 Validation Error: Transients missing frequency points",
                ));
            }

            if self.voices.transients[0].len() != self.voices.transients[1].len() {
                return Err(String::from("V0 Validation Error: Transients missing pair"));
            }

            let transients = &self.voices.transients;

            for pair in transients[0].iter().zip(transients[1].iter()) {
                if pair.0.time.is_nan() || pair.1.time.is_nan() {
                    return Err("V0 Validation Error: Transient timestamp is NaN".into());
                }

                if (pair.0.time - pair.1.time).abs() > 0.0 {
                    // check if both transients time stamps match
                    return Err(format!(
                        "V0 Validation Error: Mismatch in Transient timestamp: {} {}",
                        pair.0.time, pair.1.time
                    ));
                }

                if pair.0.amplitude > MAX_ENVELOPE_AMPLITUDE
                    || pair.1.amplitude > MAX_ENVELOPE_AMPLITUDE
                    || pair.0.amplitude < MIN_ENVELOPE_AMPLITUDE
                    || pair.1.amplitude < MIN_ENVELOPE_AMPLITUDE
                {
                    return Err(format!(
                        "V0 Validation Error: Transient amplitude out of range: {}",
                        pair.0.time,
                    ));
                }
            }
        }

        Ok(self)
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn load_file_from_test_data(path: &str) -> String {
        std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/test_data")
                .join(path),
        )
        .unwrap()
    }

    /// Helper function to generate transient data.
    fn generate_test_transients() -> Vec<Envelope> {
        vec![
            vec![
                Breakpoint {
                    time: 0.005_809_15,
                    amplitude: 1.0,
                },
                Breakpoint {
                    time: 0.113_655_4,
                    amplitude: 1.0,
                },
                Breakpoint {
                    time: 0.141_818_78,
                    amplitude: 1.0,
                },
            ],
            vec![
                Breakpoint {
                    time: 0.005_809_15,
                    amplitude: 0.858_924_9,
                },
                Breakpoint {
                    time: 0.113_655_4,
                    amplitude: 0.603_414_8,
                },
                Breakpoint {
                    time: 0.141_818_78,
                    amplitude: 0.551_285_5,
                },
            ],
        ]
    }

    /// Helper function to generate envelope data.
    fn generate_test_envelopes() -> Vec<Envelope> {
        vec![
            vec![
                Breakpoint {
                    time: 0.0,
                    amplitude: 0.0,
                },
                Breakpoint {
                    time: 0.005_809_15,
                    amplitude: 0.029_807_597,
                },
                Breakpoint {
                    time: 0.113_655_4,
                    amplitude: 0.071_484_42,
                },
                Breakpoint {
                    time: 0.141_818_78,
                    amplitude: 0.156_462_5,
                },
                Breakpoint {
                    time: 0.920_181_4,
                    amplitude: 0.112_719_33,
                },
            ],
            vec![
                Breakpoint {
                    time: 0.0,
                    amplitude: 0.0,
                },
                Breakpoint {
                    time: 0.726_530_6,
                    amplitude: 0.039_807_6,
                },
                Breakpoint {
                    time: 0.854_421_8,
                    amplitude: 0.041_484_41,
                },
                Breakpoint {
                    time: 0.877_551,
                    amplitude: 0.656_462_5,
                },
                Breakpoint {
                    time: 0.920_181_4,
                    amplitude: 0.912_719_3,
                },
            ],
        ]
    }

    /// Unit test deserializing a vij file.
    #[test]
    fn check_json_deserialize() {
        let data = load_file_from_test_data("valid_v0.vij");
        let vij: DataModel = serde_json::from_str(&data).unwrap();

        let test_version = Version {
            major: 0,
            minor: 2,
            patch: 0,
        };
        let test_metadata = MetaData {
            editor: "Lofelt GmbH".to_owned(),
            duration: 9.961_361,
        };

        let test_breakpoints = generate_test_envelopes();
        let test_transients = generate_test_transients();

        let test_voices = Voices {
            envelopes: test_breakpoints,
            transients: test_transients,
        };

        assert_eq!(vij.version, test_version);
        assert_eq!(vij.metadata, test_metadata);
        assert_eq!(vij.voices, test_voices);
    }

    /// Unit test deserializing a vij file with only transients.
    #[test]
    fn check_json_deserialize_transients() {
        let data = load_file_from_test_data("no_envelopes_v0.vij");
        let vij: DataModel = serde_json::from_str(&data).unwrap();

        let test_version = Version {
            major: 0,
            minor: 2,
            patch: 0,
        };
        let test_metadata = MetaData {
            editor: "".to_owned(),
            duration: 0.0,
        };

        let test_breakpoints = vec![];

        let test_transients = generate_test_transients();

        let test_voices = Voices {
            envelopes: test_breakpoints,
            transients: test_transients,
        };

        assert_eq!(vij.version, test_version);
        assert_eq!(vij.metadata, test_metadata);
        assert_eq!(vij.voices, test_voices);
    }

    /// Unit test failing to deserialize an incomplete vij file.
    #[test]
    fn check_json_deserialize_empty() {
        let data = load_file_from_test_data("no_voices_v0.vij");
        let err = serde_json::from_str::<DataModel>(&data)
            .map(|_| ())
            .unwrap_err();
        assert!(
            err.to_string().contains("missing field `voices`"),
            "Data model should have missing 'voices'"
        );
    }

    /// Unit test serializing and deserializing a data structure.
    #[test]
    fn check_serialize_deserialize() {
        let test_version = Version {
            major: 0,
            minor: 2,
            patch: 0,
        };
        let test_metadata = MetaData {
            editor: "Tester".to_owned(),
            duration: 1.12345,
        };

        let test_breakpoints = generate_test_envelopes();
        let test_transients = generate_test_transients();

        let test_voices = Voices {
            envelopes: test_breakpoints,
            transients: test_transients,
        };

        let test_datamodel = DataModel {
            version: test_version,
            metadata: test_metadata,
            voices: test_voices,
        };

        let serialized_data = serde_json::to_string_pretty(&test_datamodel).unwrap();
        let deserialized_data: DataModel = serde_json::from_str(&serialized_data).unwrap();

        let test_version = Version {
            major: 0,
            minor: 2,
            patch: 0,
        };
        let test_metadata = MetaData {
            editor: "Tester".to_owned(),
            duration: 1.12345,
        };

        let test_breakpoints = generate_test_envelopes();
        let test_transients = generate_test_transients();

        let test_voices = Voices {
            envelopes: test_breakpoints,
            transients: test_transients,
        };

        assert_eq!(deserialized_data.version, test_version);
        assert_eq!(deserialized_data.metadata, test_metadata);
        assert_eq!(deserialized_data.voices, test_voices);
    }

    /// Unit test datamodel validation.
    #[test]
    fn check_validation_pass() {
        let data = load_file_from_test_data("valid_v0.vij");
        serde_json::from_str::<DataModel>(&data).unwrap();
    }

    #[test]
    fn check_validation_pass_opz() {
        let data = load_file_from_test_data("invalid_v0_opz.vij");
        let vij: DataModel = serde_json::from_str(&data).unwrap();
        let err = vij.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Breakpoint amplitude out of range"),
            "Failed validation at wrong point: {}",
            err
        );
    }

    #[test]
    fn check_validation_pass_car() {
        let data = load_file_from_test_data("invalid_v0_car.vij");
        let vij: DataModel = serde_json::from_str(&data).unwrap();
        let err = vij.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Mismatch in Transient"),
            "Failed validation at wrong point: {}",
            err
        );
    }

    /// Unit test datamodel validation.
    #[test]
    fn check_validation_fail_envelopes() {
        let data = load_file_from_test_data("no_envelopes_v0.vij");
        let vij: DataModel = serde_json::from_str(&data).unwrap();
        let err = vij.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Envelopes are empty"),
            "Failed validation at wrong point: {}",
            err
        );
    }

    /// Unit test datamodel validation.
    #[test]
    fn check_validation_fail_frequency_transients() {
        let data = load_file_from_test_data("invalid_v0_conversion.vij");
        let vij: DataModel = serde_json::from_str(&data).unwrap();
        let err = vij.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Transients missing frequency point"),
            "Failed validation at wrong point: {}",
            err
        );
    }

    /// Unit test datamodel validation.
    #[test]
    fn check_validation_fail_timestamp_transients() {
        let data = load_file_from_test_data("invalid_v0_transient_mismatch.vij");
        let vij: DataModel = serde_json::from_str(&data).unwrap();
        let err = vij.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Mismatch in Transient timestamp"),
            "Failed validation at wrong point: {}",
            err
        );
    }

    /// Unit test datamodel validation.
    #[test]
    fn check_validation_fail_mismatch_transients() {
        let data = load_file_from_test_data("invalid_v0_conversions_transients.vij");
        let vij: DataModel = serde_json::from_str(&data).unwrap();
        let err = vij.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Transients missing pair"),
            "Failed validation at wrong point: {}",
            err
        );
    }
}
