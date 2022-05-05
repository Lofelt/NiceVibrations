//! Lofelt Data Model Version 1.0.0

use crate::version::{DataModelVersion, Version};
use crate::Validation;
use crate::MAX_ENVELOPE_AMPLITUDE;
use crate::MIN_ENVELOPE_AMPLITUDE;
use serde::{Deserialize, Serialize};
use typescript_definitions_local::TypescriptDefinition;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

impl DataModelVersion for DataModel {
    const CURRENT: Version = Version {
        major: 1,
        minor: 0,
        patch: 0,
    };

    fn version(&self) -> &Version {
        &self.version
    }
}

/// Main structure containing V1.0.0 of Lofelt Data Model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TypescriptDefinition)]
pub struct DataModel {
    pub version: Version,
    #[serde(default)]
    pub metadata: MetaData,
    pub signals: Signals,
}

impl Default for DataModel {
    fn default() -> Self {
        Self {
            version: Self::CURRENT,
            metadata: Default::default(),
            signals: Default::default(),
        }
    }
}

///(optional) Metadata structure
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Debug, TypescriptDefinition)]
pub struct MetaData {
    #[serde(default)]
    pub editor: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub project: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: String,
}

/// Signal structure that describes haptic data.
///
/// - A `SignalContinuous` that represents a decomposed haptic signal over a period of time (required)
///
/// A `SignalContinuous` requires an `EnvelopeAmplitude`, and can have an optional `EnvelopeFrequency`.
#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize, TypescriptDefinition)]
pub struct Signals {
    pub continuous: SignalContinuous,
}

/// Represents a decomposed haptic signal over a period of time
#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize, TypescriptDefinition)]
pub struct SignalContinuous {
    pub envelopes: Envelopes,
}

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize, TypescriptDefinition)]
/// Envelopes of a `SignalContinuous`. Allows to change `amplitude` and `frequency` of a `SignalContinuous` over time.
pub struct Envelopes {
    pub amplitude: Vec<AmplitudeBreakpoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<Vec<FrequencyBreakpoint>>,
}

/// Amplitude breakpoints of a `SignalContinuous` Amplitude envelope. Allows to apply emphasis to a point.
#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize, TypescriptDefinition)]
pub struct AmplitudeBreakpoint {
    pub time: f32,
    pub amplitude: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emphasis: Option<Emphasis>,
}

impl AmplitudeBreakpoint {
    pub fn from_interpolated_breakpoints(
        breakpoint_a: &AmplitudeBreakpoint,
        breakpoint_b: &AmplitudeBreakpoint,
        time: f32,
    ) -> Self {
        AmplitudeBreakpoint {
            time,
            amplitude: utils::interpolate(
                breakpoint_a.time,
                breakpoint_b.time,
                breakpoint_a.amplitude,
                breakpoint_b.amplitude,
                time,
            ),
            emphasis: None,
        }
    }
}

/// Emphasis structure associated with a Amplitude envelope breakpoint. Allows for a "haptic highlight" of the breakpoint.
#[derive(Clone, Copy, Default, PartialEq, Debug, Serialize, Deserialize, TypescriptDefinition)]
#[repr(C)]
pub struct Emphasis {
    pub amplitude: f32,
    pub frequency: f32,
}

/// Data associated with a Frequency envelope breakpoint.
#[derive(Clone, Copy, Default, PartialEq, Debug, Serialize, Deserialize, TypescriptDefinition)]
pub struct FrequencyBreakpoint {
    pub time: f32,
    pub frequency: f32,
}

impl FrequencyBreakpoint {
    pub fn from_interpolated_breakpoints(
        breakpoint_a: &FrequencyBreakpoint,
        breakpoint_b: &FrequencyBreakpoint,
        time: f32,
    ) -> Self {
        FrequencyBreakpoint {
            time,
            frequency: utils::interpolate(
                breakpoint_a.time,
                breakpoint_b.time,
                breakpoint_a.frequency,
                breakpoint_b.frequency,
                time,
            ),
        }
    }
}

impl DataModel {
    /// Removes all breakpoints before the specified `time` (in seconds) from the DataModel.
    ///
    /// The time of all remaining breakpoints is shifted so that the new first breakpoint starts
    /// at 0.0. That means the total duration is reduced by `time`.
    ///
    /// If truncation happens to occur at exactly the same time as an existing breakpoint, no new
    /// initial breakpoint needs to be inserted.
    /// Otherwise a new initial breakpoint is inserted with time 0.0, as otherwise the first
    /// breakpoint would start later, and playback wouldn't begin at 0.0. The amplitude and
    /// frequency of this new first breakpoint is an interpolation of the amplitude/frequency of its
    /// neighboring breakpoints.
    pub fn truncate_before(&mut self, time: f32) -> Result<(), String> {
        //
        // Truncate amplitude
        //
        let amplitudes = &mut self.signals.continuous.envelopes.amplitude;
        let index_of_first_breakpoint_in_range = amplitudes
            .iter()
            .position(|breakpoint| breakpoint.time >= time);

        if index_of_first_breakpoint_in_range.is_none() {
            return Err("No amplitude breakpoint before the specified starting time".to_string());
        }

        let index_of_first_breakpoint_in_range = index_of_first_breakpoint_in_range.unwrap();
        if index_of_first_breakpoint_in_range > 0 {
            let breakpoint_before = &amplitudes[index_of_first_breakpoint_in_range - 1];
            let breakpoint_after = &amplitudes[index_of_first_breakpoint_in_range];
            let new_first_breakpoint =
                if breakpoint_after.time - breakpoint_before.time > f32::EPSILON {
                    Some(AmplitudeBreakpoint {
                        time: 0.0,
                        amplitude: utils::interpolate(
                            breakpoint_before.time,
                            breakpoint_after.time,
                            breakpoint_before.amplitude,
                            breakpoint_after.amplitude,
                            time,
                        ),
                        emphasis: None,
                    })
                } else {
                    None
                };

            // Remove breakpoints before `time`
            amplitudes.retain(|breakpoint| breakpoint.time >= time);

            // Shift the time of all breakpoints by `time`
            for breakpoint in amplitudes.iter_mut() {
                breakpoint.time -= time;
            }

            // Insert a new first breakpoint
            if let Some(new_first_breakpoint) = new_first_breakpoint {
                amplitudes.insert(0, new_first_breakpoint);
            }
        }

        //
        // Truncate frequency
        // Same algorithm as for the amplitude, except that the frequency envelope is optional.
        //
        let frequencies = &mut self.signals.continuous.envelopes.frequency;
        if let Some(frequencies) = frequencies {
            let index_of_first_breakpoint_in_range = frequencies
                .iter()
                .position(|breakpoint| breakpoint.time >= time);

            if let Some(index_of_first_breakpoint_in_range) = index_of_first_breakpoint_in_range {
                if index_of_first_breakpoint_in_range > 0 {
                    let breakpoint_before = &frequencies[index_of_first_breakpoint_in_range - 1];
                    let breakpoint_after = &frequencies[index_of_first_breakpoint_in_range];
                    let new_first_breakpoint =
                        if breakpoint_after.time - breakpoint_before.time > f32::EPSILON {
                            Some(FrequencyBreakpoint {
                                time: 0.0,
                                frequency: utils::interpolate(
                                    breakpoint_before.time,
                                    breakpoint_after.time,
                                    breakpoint_before.frequency,
                                    breakpoint_after.frequency,
                                    time,
                                ),
                            })
                        } else {
                            None
                        };
                    frequencies.retain(|breakpoint| breakpoint.time >= time);
                    for breakpoint in frequencies.iter_mut() {
                        breakpoint.time -= time;
                    }
                    if let Some(new_first_breakpoint) = new_first_breakpoint {
                        frequencies.insert(0, new_first_breakpoint);
                    }
                }
            } else {
                self.signals.continuous.envelopes.frequency = None;
            }
        }

        Ok(())
    }
}

/// Validation trait implementation
/// An invalid Data Model would be one that:
/// - Breakpoints and emphasis values are < 0.0 or > 1.0.
/// - The breakpoint time values are not consecutive.
/// - Emphasis amplitude is smaller than breakpoint amplitude value
impl Validation for DataModel {
    fn validate(self) -> Result<Self, String> {
        let mut last_time: f32 = 0.0; // variable to keep track of the previous breakpoint time

        if self.signals.continuous.envelopes.amplitude.is_empty() {
            return Err(String::from(
                "V1 Validation Error: Amplitude envelope is empty",
            ));
        }

        for amplitude_envelope in self.signals.continuous.envelopes.amplitude.iter() {
            if amplitude_envelope.amplitude < MIN_ENVELOPE_AMPLITUDE
                || amplitude_envelope.amplitude > MAX_ENVELOPE_AMPLITUDE
            {
                return Err(format!(
                    "V1 Validation Error: Breakpoint amplitude out of range: {}",
                    amplitude_envelope.time,
                ));
            }

            if last_time > amplitude_envelope.time {
                return Err(format!(
                    "V1 Validation Error: Breakpoint times not consecutive: {} after {}",
                    amplitude_envelope.time, last_time,
                ));
            }

            last_time = amplitude_envelope.time;

            if let Some(emphasis) = &amplitude_envelope.emphasis {
                if emphasis.amplitude > MAX_ENVELOPE_AMPLITUDE
                    || emphasis.amplitude < MIN_ENVELOPE_AMPLITUDE
                {
                    return Err(format!(
                        "V1 Validation Error: Emphasis amplitude out of range: {}",
                        emphasis.amplitude,
                    ));
                }

                if emphasis.frequency > MAX_ENVELOPE_AMPLITUDE
                    || emphasis.frequency < MIN_ENVELOPE_AMPLITUDE
                {
                    return Err(format!(
                        "V1 Validation Error: Emphasis frequency out of range: {}",
                        emphasis.frequency,
                    ));
                }

                if emphasis.amplitude < amplitude_envelope.amplitude {
                    return Err(format!(
                        "V1 Validation: Emphasis amplitude can't be lower than Envelope amplitude:
                        {} smaller than {} at {}",
                        emphasis.amplitude, amplitude_envelope.amplitude, amplitude_envelope.time
                    ));
                }
            }
        }

        if let Some(frequency_envelopes) = &self.signals.continuous.envelopes.frequency {
            last_time = 0.0;
            for frequency_envelope in frequency_envelopes.iter() {
                if frequency_envelope.frequency < MIN_ENVELOPE_AMPLITUDE
                    || frequency_envelope.frequency > MAX_ENVELOPE_AMPLITUDE
                {
                    return Err(format!(
                        "V1 Validation Error: Breakpoint frequency out of range: {}",
                        frequency_envelope.time,
                    ));
                }

                if last_time > frequency_envelope.time {
                    return Err(format!(
                        "V1 Validation Error: Breakpoint frequency times not consecutive: {} after {}",
                        frequency_envelope.time, last_time,
                    ));
                }

                last_time = frequency_envelope.time;
            }
        }

        Ok(self)
    }
}

fn add_v0_transients_to_v1_breakpoints(
    mut v0_transients: Vec<crate::v0::Envelope>,
    v1_amplitude_breakpoints: &mut [AmplitudeBreakpoint],
) {
    if v0_transients.len() != 2 || v0_transients[0].len() != v0_transients[1].len() {
        return;
    }

    // Iterate over all amplitude breakpoints and check if there is a transient at the same
    // timestamp. If that's the case, convert the transient to emphasis and add it to the
    // amplitude breakpoint.
    // Transients that don't have a matching amplitude breakpoint at the same
    // timestamp are silently ignored. It would be possible to insert a new amplitude breakpoint
    // with such a timestamp, but since v0 is an old format and such transients can probably not
    // be found in the wild, it's not worth the effort.
    v1_amplitude_breakpoints
        .iter_mut()
        .for_each(|v1_amplitude_breakpoint| {
            if let Ok(v0_transient_index) = v0_transients[0].binary_search_by(|v0_transient| {
                v0_transient
                    .time
                    .partial_cmp(&v1_amplitude_breakpoint.time)
                    .unwrap()
            }) {
                let v0_transient_amplitude = v0_transients[0][v0_transient_index].amplitude;
                let v0_transient_frequency = v0_transients[1][v0_transient_index].amplitude;
                v1_amplitude_breakpoint.emphasis = Some(Emphasis {
                    amplitude: v0_transient_amplitude,
                    frequency: v0_transient_frequency,
                });

                v0_transients[0].remove(v0_transient_index);
                v0_transients[1].remove(v0_transient_index);
            }
        });
}

/// Implementation of upgrade functionality from version V0.
impl From<crate::v0::DataModel> for crate::v1::DataModel {
    fn from(v0: crate::v0::DataModel) -> Self {
        let version: Version = DataModel::CURRENT;
        let mut signals = Signals::default();

        // The first array of breakpoints is mapped to amplitude.
        let mut amplitude_envelopes: Vec<AmplitudeBreakpoint> = v0.voices.envelopes[0]
            .iter()
            .map(|breakpoint| AmplitudeBreakpoint {
                time: breakpoint.time,
                amplitude: breakpoint.amplitude,
                emphasis: None,
            })
            .collect();

        // add a last point to the continuous amplitude envelope, corresponding to the
        // duration of the signal
        let event_amplitude_to_add = match amplitude_envelopes.last() {
            Some(last_event) => {
                if v0.metadata.duration > last_event.time {
                    Some(last_event.amplitude)
                } else {
                    None
                }
            }
            None => Some(0.0),
        };

        if let Some(amplitude) = event_amplitude_to_add {
            amplitude_envelopes.push(AmplitudeBreakpoint {
                time: v0.metadata.duration,
                amplitude,
                emphasis: None,
            });
        }

        // The second array of breakpoints is mapped to frequency.
        let frequency_envelopes: Vec<FrequencyBreakpoint> = if v0.voices.envelopes.len() == 2 {
            v0.voices.envelopes[1]
                .iter()
                .map(|breakpoint| FrequencyBreakpoint {
                    time: breakpoint.time,
                    frequency: breakpoint.amplitude,
                })
                .collect()
        } else {
            vec![]
        };

        add_v0_transients_to_v1_breakpoints(v0.voices.transients, &mut amplitude_envelopes);

        // The only thing common in Metadata is the editor field.
        let metadata = MetaData {
            editor: v0.metadata.editor,
            ..Default::default()
        };

        // Assign the amplitude envelopes to our signals struct.
        signals.continuous.envelopes.amplitude = amplitude_envelopes;

        // Add frequency envelopes if present.
        if !frequency_envelopes.is_empty() {
            signals.continuous.envelopes.frequency = Some(frequency_envelopes);
        } else {
            signals.continuous.envelopes.frequency = None;
        }

        // Return the updated data model structure.
        DataModel {
            version,
            metadata,
            signals,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::latest_from_json;

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

    pub fn latest_from_test_data(path: &str) -> DataModel {
        let clip_json = load_file_from_test_data(path);
        latest_from_json(&clip_json).unwrap().1
    }

    fn load_test_file_valid_required_v1() -> String {
        load_file_from_test_data("valid_required_v1.haptic")
    }

    #[test]
    fn check_test_json_deserialized_required_fields_only() {
        let data: DataModel = serde_json::from_str(&load_test_file_valid_required_v1()).unwrap();

        let metadata = MetaData::default();
        let version = Version {
            major: 1,
            minor: 0,
            patch: 0,
        };

        //check if value of data not included in the file is the default
        assert_eq!(metadata, data.metadata);
        assert_eq!(version, data.version);
        assert_eq!(data.signals.continuous.envelopes.frequency, None);
    }

    #[test]
    fn check_serialized_required_only() {
        let reference_data: DataModel =
            serde_json::from_str(&load_test_file_valid_required_v1()).unwrap();

        let metadata = MetaData::default();
        let version = Version {
            major: 1,
            minor: 0,
            patch: 0,
        };

        let amplitude_envelope = vec![
            AmplitudeBreakpoint {
                time: 0.0,
                amplitude: 0.2,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 0.1,
                amplitude: 0.3,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 0.2,
                amplitude: 0.2,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 0.3,
                amplitude: 0.5,
                emphasis: None,
            },
        ];

        let signal_continuous = SignalContinuous {
            envelopes: Envelopes {
                amplitude: amplitude_envelope,
                frequency: None,
            },
        };

        let data = DataModel {
            version,
            metadata,
            signals: Signals {
                continuous: signal_continuous,
            },
        };

        assert_eq!(reference_data, data);
    }

    #[test]
    fn check_test_json_deserialize() {
        let data: DataModel =
            serde_json::from_str(&load_file_from_test_data("valid_v1.haptic")).unwrap();

        let version = Version {
            major: 1,
            minor: 0,
            patch: 0,
        };

        //check if value of data not included in the file is the default
        assert_eq!(version, data.version);
    }

    #[test]
    fn check_test_json_deserialize_invalid_fields() {
        let data = serde_json::from_str::<DataModel>(&load_file_from_test_data(
            "invalid_fields_v1.haptic",
        ));
        let err = data.map(|_| ()).unwrap_err();
        assert!(err.to_string().contains("missing field `signals`"));
    }

    pub fn create_test_data_model() -> DataModel {
        //building data
        let version: Version = Version {
            major: 1,
            minor: 0,
            patch: 0,
        };

        let metadata = MetaData {
            editor: "VSCode".to_owned(),
            author: "SDK Team".to_owned(),
            tags: vec!["Test".to_owned()],
            description: "Testing".to_owned(),
            ..Default::default()
        };

        let envelope_amplitude = vec![
            AmplitudeBreakpoint {
                time: 0.0,
                amplitude: 0.2,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 0.1,
                amplitude: 0.3,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 0.2,
                amplitude: 0.2,
                emphasis: None,
            },
            AmplitudeBreakpoint {
                time: 0.3,
                amplitude: 0.5,
                emphasis: Some(Emphasis {
                    amplitude: 0.69,
                    frequency: 0.7,
                }),
            },
        ];

        let envelope_frequency = vec![
            FrequencyBreakpoint {
                time: 0.1,
                frequency: 0.99,
            },
            FrequencyBreakpoint {
                time: 0.2,
                frequency: 0.54,
            },
            FrequencyBreakpoint {
                time: 0.25,
                frequency: 0.8,
            },
            FrequencyBreakpoint {
                time: 0.3,
                frequency: 0.9,
            },
        ];

        let signal_continuous = SignalContinuous {
            envelopes: Envelopes {
                amplitude: envelope_amplitude,
                frequency: Some(envelope_frequency),
            },
        };

        DataModel {
            version,
            metadata,
            signals: Signals {
                continuous: signal_continuous,
            },
        }
    }

    fn serialize_test_data_json() -> String {
        let data = create_test_data_model();
        serde_json::to_string_pretty(&data).unwrap()
    }

    fn deserialize_test_data_json() -> DataModel {
        let serialized_json = serialize_test_data_json();
        let deserialized_json: DataModel = serde_json::from_str(&serialized_json).unwrap();

        deserialized_json
    }

    #[test]
    fn check_test_json_serialize_deserialize() {
        //verify if deserialized data matches the created data to be serialized
        let deserialized_json = deserialize_test_data_json();

        //version
        assert_eq!(deserialized_json.version.major, 1);
        assert_eq!(deserialized_json.version.minor, 0);
        assert_eq!(deserialized_json.version.patch, 0);

        //metadata
        assert_eq!(deserialized_json.metadata.author, "SDK Team");
        assert_eq!(deserialized_json.metadata.description, "Testing");
        assert_eq!(deserialized_json.metadata.editor, "VSCode");
        assert_eq!(deserialized_json.metadata.tags[0], "Test");

        //signals
        let serialized_signals = deserialized_json.signals;

        // check continuous

        assert_eq!(
            serialized_signals.continuous.envelopes.amplitude[0],
            AmplitudeBreakpoint {
                time: 0.0,
                amplitude: 0.2,
                emphasis: None
            }
        );
        assert_eq!(
            serialized_signals.continuous.envelopes.amplitude[1],
            AmplitudeBreakpoint {
                time: 0.1,
                amplitude: 0.3,
                emphasis: None
            }
        );
        assert_eq!(
            serialized_signals.continuous.envelopes.amplitude[2],
            AmplitudeBreakpoint {
                time: 0.2,
                amplitude: 0.2,
                emphasis: None
            }
        );
        assert_eq!(
            serialized_signals.continuous.envelopes.amplitude[3],
            AmplitudeBreakpoint {
                time: 0.3,
                amplitude: 0.5,
                emphasis: Some(Emphasis {
                    amplitude: 0.69,
                    frequency: 0.7,
                }),
            }
        );

        let freq_vec = serialized_signals.continuous.envelopes.frequency.unwrap();
        assert_eq!(
            freq_vec[0],
            FrequencyBreakpoint {
                time: 0.1,
                frequency: 0.99
            }
        );
        assert_eq!(
            freq_vec[1],
            FrequencyBreakpoint {
                time: 0.2,
                frequency: 0.54
            }
        );
        assert_eq!(
            freq_vec[2],
            FrequencyBreakpoint {
                time: 0.25,
                frequency: 0.8
            }
        );
        assert_eq!(
            freq_vec[3],
            FrequencyBreakpoint {
                time: 0.3,
                frequency: 0.9
            }
        );
    }

    /// Utility function to check v0 to v1 version upgrading
    fn check_v0_to_v1_upgrade(v0_file_name: &str, v1_file_name: &str, validate_v0: bool) {
        let v0: crate::v0::DataModel =
            serde_json::from_str(&load_file_from_test_data(v0_file_name)).unwrap();

        let v0 = if validate_v0 {
            v0.validate().unwrap()
        } else {
            v0
        };

        let v1 = crate::v1::DataModel::from(v0);

        let v1_validation: crate::v1::DataModel =
            serde_json::from_str(&load_file_from_test_data(v1_file_name)).unwrap();
        assert_eq!(v1.version, v1_validation.version);
        assert_eq!(v1.signals, v1_validation.signals);
    }

    /// unit test to check version upgrading.
    #[test]
    fn check_version_upgrade() {
        check_v0_to_v1_upgrade("valid_v0_conversion.vij", "valid_v1_from_v0.haptic", true);
    }

    // Unit to to check v0 to v1 upgrade on a real-world file produced by the DSP code.
    // All transients in that file are valid.
    #[test]
    fn check_version_upgrade_v0_from_dsp() {
        check_v0_to_v1_upgrade(
            "valid_v0_from_dsp.vij",
            "valid_v1_from_v0_from_dsp.haptic",
            true,
        );
    }

    // Unit test to check v0 to v1 upgrade. The v0 file has one valid transient
    // and one transient without a matching amplitude breakpoint at the same timestamp.
    // While that's a valid v0 file, we ignore that transient in the upgrade.
    #[test]
    fn check_version_upgrade_transient_amplitude_breakpoint_mismatch() {
        check_v0_to_v1_upgrade(
            "valid_v0_transient_time_mismatch.vij",
            "valid_v1_from_v0_transient_time_mismatch.haptic",
            true,
        );
    }

    // unit test to check version upgrading ignoring incorrect transients.
    #[test]
    fn check_version_upgrade_transients() {
        check_v0_to_v1_upgrade(
            "invalid_v0_conversions_transients.vij",
            "valid_v1_from_invalid_v0_conversions_transients.haptic",
            false,
        );
    }

    /// unit test to check version upgrading ignoring incorrect transients and frequency_envelopes.
    #[test]
    fn check_version_upgrade_invalid() {
        check_v0_to_v1_upgrade(
            "invalid_v0_conversion.vij",
            "valid_v1_from_invalid_v0_conversion.haptic",
            false,
        );
    }

    /// Unit test datamodel validation.
    #[test]
    fn check_validation_pass() {
        let data = load_file_from_test_data("valid_v1.haptic");
        let data: DataModel = serde_json::from_str(&data).unwrap();
        data.validate().unwrap();
    }

    /// Unit test datamodel validation optionals.
    #[test]
    fn check_validation_optional() {
        let data = load_file_from_test_data("validation_v1_optionals.haptic");
        let data: DataModel = serde_json::from_str(&data).unwrap();
        data.validate().unwrap();
    }

    /// Unit test datamodel validation amplitude range.
    #[test]
    fn check_validation_fail_range() {
        let data = load_file_from_test_data("validation_v1_amplitude.haptic");
        let data: DataModel = serde_json::from_str(&data).unwrap();
        let err = data.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Breakpoint amplitude out of range"),
            "Failed validation at wrong point: {}",
            err
        );
    }

    /// Unit test datamodel validation consecutive breakpoints.
    #[test]
    fn check_validation_fail_sequence() {
        let data = load_file_from_test_data("validation_v1_sequence.haptic");
        let data: DataModel = serde_json::from_str(&data).unwrap();
        let err = data.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Breakpoint times not consecutive"),
            "Failed validation at wrong point: {}",
            err
        );
    }

    #[test]
    fn check_validation_fail_emphasis_amplitude_vs_signal_amplitude() {
        let data = load_file_from_test_data("validation_v1_emphasis_amplitude.haptic");
        let data: DataModel = serde_json::from_str(&data).unwrap();
        let err = data.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Emphasis amplitude can't be lower than Envelope amplitude"),
            "Failed validation with wrong message: {}",
            err
        );
    }

    #[test]
    fn check_validation_fail_emphasis_amplitude_range() {
        let data = load_file_from_test_data("validation_v1_emphasis_amplitude_range.haptic");
        let haptic: DataModel = serde_json::from_str(&data).unwrap();
        let err = haptic.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Emphasis amplitude out of range"),
            "Failed validation with wrong message: {}",
            err
        );
    }

    #[test]
    fn check_validation_fail_emphasis_frequency_range() {
        let data = load_file_from_test_data("validation_v1_emphasis_frequency_range.haptic");
        let haptic: DataModel = serde_json::from_str(&data).unwrap();
        let err = haptic.validate().map(|_| ()).unwrap_err();
        assert!(
            err.contains("Emphasis frequency out of range"),
            "Failed validation with wrong message: {}",
            err
        );
    }

    #[test]
    fn check_valid_beta_impulses() {
        let data: String = load_file_from_test_data("valid_beta_impulses.haptic");
        let haptic: DataModel = serde_json::from_str(&data).unwrap();
        haptic.validate().unwrap();
    }

    #[test]
    // Test that truncating before a value works as expected
    fn truncate() {
        let mut before_truncate = latest_from_test_data("truncate_before.haptic");
        let after_truncate = latest_from_test_data("truncate_after.haptic");
        before_truncate.truncate_before(2.5).unwrap();
        assert_eq!(before_truncate.signals, after_truncate.signals);
    }

    #[test]
    // Test that truncating before a value after the end of the clip returns an error
    fn truncate_after_end() {
        let mut before_truncate = latest_from_test_data("truncate_before.haptic");
        assert_eq!(
            before_truncate.truncate_before(100.0),
            Err("No amplitude breakpoint before the specified starting time".to_string())
        );
    }

    #[test]
    // Truncating with just 2 breakpoints
    fn truncate_2_breakpoints() {
        let mut before_truncate = latest_from_test_data("truncate_before_2_bp.haptic");
        let after_truncate = latest_from_test_data("truncate_after_2_bp.haptic");
        before_truncate.truncate_before(0.5).unwrap();
        assert_eq!(before_truncate.signals, after_truncate.signals);
    }

    #[test]
    // Truncating with 1 breakpoint fails
    fn truncate_1_breakpoint() {
        let mut before_truncate = latest_from_test_data("truncate_before_1_bp.haptic");
        assert_eq!(
            before_truncate.truncate_before(1.0),
            Err("No amplitude breakpoint before the specified starting time".to_string())
        );
    }

    #[test]
    // Truncating with empty frequency
    fn truncate_empty_frequency_envelope_before() {
        // empty frequency envelope before truncating
        let mut before_truncate =
            latest_from_test_data("truncate_with_empty_frequency_before.haptic");
        let after_truncate =
            latest_from_test_data("truncate_after_with_empty_frequency_before.haptic");
        before_truncate.truncate_before(2.5).unwrap();
        assert_eq!(before_truncate.signals, after_truncate.signals);
    }
    #[test]
    // Truncating results in a empty frequency envelope
    fn truncate_empty_frequency_envelope_after() {
        // empty frequency envelope before truncating
        let mut before_truncate =
            latest_from_test_data("truncate_with_empty_frequency_after.haptic");
        let after_truncate =
            latest_from_test_data("truncate_after_with_empty_frequency_after.haptic");

        before_truncate.truncate_before(2.5).unwrap();
        assert_eq!(before_truncate.signals, after_truncate.signals);
    }
}
