// Copyright (c) Meta Platforms, Inc. and affiliates.

//! Defines the iOS data model version 1.0.0
use crate::*;
use serde::{Deserialize, Serialize};
use std::f32;
use v1::AmplitudeBreakpoint;

const DELTA_ERR: f32 = 0.000_000_1;
const AMPLITUDE_DUCKING: f32 = 0.2;

const MAX_CONTINUOUS_EVENT_DURATION: f32 = 30.0;

///Core Haptics AHAP data model structure
#[derive(Default, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Ahap {
    pub version: f32,
    pub metadata: MetaData,
    pub pattern: Vec<Pattern>,
}

impl Ahap {
    ///Converts AHAP data into a AHAP string
    pub fn to_string(ahap_data: &Ahap) -> Result<String, String> {
        match serde_json::to_string::<Ahap>(ahap_data) {
            Ok(ahap_string) => Ok(ahap_string),
            Err(e) => Err(e.to_string()),
        }
    }

    ///Converts AHAP data into a AHAP string pretty
    pub fn to_string_pretty(ahap_data: &Ahap) -> Result<String, String> {
        match serde_json::to_string_pretty::<Ahap>(ahap_data) {
            Ok(ahap_string) => Ok(ahap_string),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Splits AHAP data into two AHAPs with continuous and transient events respectively
    pub fn into_continuous_and_transients_ahaps(self) -> (Ahap, Option<Ahap>) {
        let mut ahap_transients = Ahap::default();
        let mut ahap_continuous = Ahap::default();

        for pattern in self.pattern {
            match pattern {
                Pattern::Event(event) => match event {
                    Event::HapticContinuous {
                        time,
                        event_duration,
                        event_parameters,
                    } => ahap_continuous
                        .pattern
                        .push(Pattern::Event(Event::HapticContinuous {
                            time,
                            event_duration,
                            event_parameters,
                        })),
                    Event::HapticTransient {
                        time,
                        event_parameters,
                    } => ahap_transients
                        .pattern
                        .push(Pattern::Event(Event::HapticTransient {
                            time,
                            event_parameters,
                        })),
                },
                Pattern::ParameterCurve(parameter_curve) => {
                    ahap_continuous
                        .pattern
                        .push(Pattern::ParameterCurve(parameter_curve));
                }
            }
        }

        //in case there are no transients in AHAP return None so they are not played
        if ahap_transients.pattern.is_empty() {
            (ahap_continuous, None)
        } else {
            (ahap_continuous, Some(ahap_transients))
        }
    }
}

fn ahap_transient_events_from_breakpoints(breakpoints: &[AmplitudeBreakpoint]) -> Vec<Pattern> {
    breakpoints
        .iter()
        .filter(|&x| x.emphasis.is_some())
        .map(|x| {
            Pattern::Event(Event::HapticTransient {
                time: x.time,
                event_parameters: vec![
                    EventParameter {
                        parameter_id: ParameterId::HapticIntensity,
                        parameter_value: x.emphasis.as_ref().map_or(0.0, |x| x.amplitude.sqrt()),
                    },
                    EventParameter {
                        parameter_id: ParameterId::HapticSharpness,
                        parameter_value: x.emphasis.as_ref().map_or(0.0, |x| x.frequency),
                    },
                ],
            })
        })
        .collect::<Vec<Pattern>>()
}

/// Creates events of type HapticContinuous for a haptic clip.
///
/// Each event will just have a constant intensity of 1 and a constant sharpness
/// of 0. The intensity and sharpness change during playback because parameter curves
/// that modulate these constant values are added to the AHAP in another place.
///
/// The only reason to use multiple events here is because CoreHaptics limits events
/// of type HapticContinuous to 30 seconds.
fn ahap_continuous_events_from_v1(clip: &v1::DataModel) -> Vec<Pattern> {
    let mut total_remaining_duration = match clip.signals.continuous.envelopes.amplitude.last() {
        None => 0.0,
        Some(last) => last.time,
    };
    let event_count = (total_remaining_duration / MAX_CONTINUOUS_EVENT_DURATION).ceil() as u32;
    let mut result = Vec::new();
    for i in 0..event_count {
        let time = i as f32 * MAX_CONTINUOUS_EVENT_DURATION;
        let event_duration = if total_remaining_duration > MAX_CONTINUOUS_EVENT_DURATION {
            MAX_CONTINUOUS_EVENT_DURATION
        } else {
            total_remaining_duration
        };
        total_remaining_duration -= event_duration;

        let ahap_pattern_continuous_event = Pattern::Event(Event::HapticContinuous {
            time,
            event_duration,
            event_parameters: vec![
                EventParameter {
                    parameter_id: ParameterId::HapticIntensity,
                    parameter_value: 1.0,
                },
                EventParameter {
                    parameter_id: ParameterId::HapticSharpness,
                    parameter_value: 0.0,
                },
            ],
        });

        result.push(ahap_pattern_continuous_event);
    }
    result
}

///Creates an AHAP data structure with data from Lofelt Data V1.0.0
impl From<v1::DataModel> for Ahap {
    fn from(v1: v1::DataModel) -> Self {
        let ahap_version = 1.0;

        let v1_signals = &v1.signals;

        // ----------------------------------------------------------------
        // CHParameterCurve Intensity from Continuous Amplitude Envelope
        // ----------------------------------------------------------------

        // get first point
        let default_control_point = v1::AmplitudeBreakpoint::default();
        let mut control_point = match v1_signals.continuous.envelopes.amplitude.first() {
            None => &default_control_point,
            Some(first) => first,
        };

        //init ahap struct where converted data from v1 will be pushed to
        let mut ahap_data = Self::default();
        //init empty transients events array
        let mut transient_events_data = Vec::new();
        // skip first element as it is already in mut control_point
        let continue_envelope_amplitude_vec = &v1_signals.continuous.envelopes.amplitude[1..];

        for amplitude_breakpoint_chunks in continue_envelope_amplitude_vec.chunks(15) {
            //first point in the CHParameterCurve comes from control_point
            let mut parameter_curve_control_points = vec![ParameterCurveControlPoint {
                time: control_point.time,
                parameter_value: get_intensity_from_amplitude_bp(control_point),
            }];

            //Add remaining 15 control points
            parameter_curve_control_points.extend(
                &amplitude_breakpoint_chunks
                    .iter()
                    .map(|point| ParameterCurveControlPoint {
                        time: point.time,
                        parameter_value: get_intensity_from_amplitude_bp(point),
                    })
                    .collect::<Vec<ParameterCurveControlPoint>>(),
            );

            // creating the parameter curve with the necessary fields
            let parameter_curve_intensity = Pattern::ParameterCurve(ParameterCurve {
                parameter_id: DynamicParameterId::HapticIntensityControl,
                time: control_point.time,
                parameter_curve_control_points,
            });

            //getting the last control point to be repeated in the first point of the next
            //CHParameterCurve
            control_point = match amplitude_breakpoint_chunks.last() {
                Some(last) => last,
                None => &default_control_point,
            };

            //adding an intensity parameter curve to Pattern Vector.
            ahap_data.pattern.push(parameter_curve_intensity);

            //getting CHTransient events if there are continuous amplitude breakpoints with emphasis
            transient_events_data.extend(ahap_transient_events_from_breakpoints(
                amplitude_breakpoint_chunks,
            ));
        }

        // ----------------------------------------------------------------
        // CHParameterCurve Sharpness from Continuous Frequency Envelope
        // ----------------------------------------------------------------

        match &v1_signals.continuous.envelopes.frequency {
            None => {}
            Some(frequency_breakpoint_vec) => {
                // get first point
                let default_control_point = v1::FrequencyBreakpoint::default();
                let mut control_point = match frequency_breakpoint_vec.first() {
                    None => &default_control_point,
                    Some(first) => first,
                };
                // skip first element as it is already in mut control_point
                let frequency_breakpoint_sliced = &frequency_breakpoint_vec[1..];

                for time_frequency_chunks in frequency_breakpoint_sliced.chunks(15) {
                    //first point in the CHParameterCurve comes from control_point
                    let mut parameter_curve_control_points = vec![ParameterCurveControlPoint {
                        time: control_point.time,
                        parameter_value: control_point.frequency.sqrt(),
                    }];

                    //Appending remaining 15 control points
                    parameter_curve_control_points.extend(
                        time_frequency_chunks
                            .iter()
                            .map(|point| ParameterCurveControlPoint {
                                time: point.time,
                                parameter_value: point.frequency.sqrt(),
                            })
                            .collect::<Vec<ParameterCurveControlPoint>>(),
                    );

                    // creating the parameter curve with the necessary fields
                    let parameter_curve_sharpness = Pattern::ParameterCurve(ParameterCurve {
                        parameter_id: DynamicParameterId::HapticSharpnessControl,
                        time: control_point.time,
                        parameter_curve_control_points,
                    });

                    //getting the last control point to be repeated in the first point of the next
                    //CHParameterCurve
                    control_point = match time_frequency_chunks.last() {
                        Some(last) => last,
                        None => &default_control_point,
                    };

                    //adding a sharpness parameter curve to Pattern Vector.
                    ahap_data.pattern.push(parameter_curve_sharpness);
                }
            }
        };

        ahap_data
            .pattern
            .append(&mut ahap_continuous_events_from_v1(&v1));

        //Appending transients at the end of AHAP to make AHAPs more organized
        ahap_data.pattern.append(&mut transient_events_data);

        ahap_data.metadata = MetaData {
            project: v1.metadata.project,
            created: v1.metadata.author,
            description: v1.metadata.description,
        };
        ahap_data.version = ahap_version;

        ahap_data
    }
}

fn get_intensity_from_amplitude_bp(breakpoint: &AmplitudeBreakpoint) -> f32 {
    if breakpoint.emphasis.is_some() {
        breakpoint.amplitude.sqrt() * (1.0 - AMPLITUDE_DUCKING)
    } else {
        breakpoint.amplitude.sqrt()
    }
}

///Core Haptics AHAP Metadata structure
#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MetaData {
    #[serde(default)]
    pub project: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub description: String,
}

///Core Haptics AHAP Pattern types
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Pattern {
    Event(Event),
    ParameterCurve(ParameterCurve),
}

///Core Haptics AHAP Event structures for `HapticContinuous` and `HapticTransient` events
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(tag = "EventType")]
pub enum Event {
    #[serde(rename_all = "PascalCase")]
    HapticContinuous {
        time: f32,
        event_duration: f32,
        event_parameters: Vec<EventParameter>,
    },
    #[serde(rename_all = "PascalCase")]
    HapticTransient {
        time: f32,
        event_parameters: Vec<EventParameter>,
    },
}

///Core Haptics AHAP EventParameter data structure
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EventParameter {
    #[serde(rename = "ParameterID")]
    pub parameter_id: ParameterId,
    pub parameter_value: f32,
}

impl PartialEq for EventParameter {
    fn eq(&self, other: &Self) -> bool {
        if self.parameter_id == other.parameter_id {
            (self.parameter_value - other.parameter_value).abs() <= DELTA_ERR
        } else {
            false
        }
    }
}

///Core Haptics AHAP Parameter data structure
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Parameter {
    #[serde(rename = "ParameterID")]
    pub parameter_id: DynamicParameterId,
    pub parameter_value: f32,
    pub time: f32,
}

///Core Haptics AHAP ParameterCurve data structure
#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ParameterCurve {
    #[serde(rename = "ParameterID")]
    pub parameter_id: DynamicParameterId,
    pub time: f32,
    pub parameter_curve_control_points: Vec<ParameterCurveControlPoint>,
}

///Core Haptics AHAP DynamicParameter data structure used in ParameterCurves
#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DynamicParameterId {
    #[default]
    HapticIntensityControl,
    HapticSharpnessControl,
}

///Core Haptics AHAP ParameterId used to describe the Event type.
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ParameterId {
    HapticIntensity,
    HapticSharpness,
}

///Core Haptics AHAP ParameterCurve control point structure
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ParameterCurveControlPoint {
    pub time: f32,
    pub parameter_value: f32,
}

impl PartialEq for ParameterCurveControlPoint {
    fn eq(&self, other: &Self) -> bool {
        if (self.time - other.time).abs() <= DELTA_ERR {
            (self.parameter_value - other.parameter_value).abs() <= DELTA_ERR
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, path::Path};

    ///Testing AHAP serialization and deserialization
    #[test]
    fn test_serializing_deserializing() {
        let metadata = MetaData::default();

        let event_parameter_intensity = EventParameter {
            parameter_id: ParameterId::HapticIntensity,
            parameter_value: 1.0,
        };
        let event_parameter_sharpness = EventParameter {
            parameter_id: ParameterId::HapticSharpness,
            parameter_value: 1.0,
        };
        let event_parameters = vec![event_parameter_intensity, event_parameter_sharpness];

        let event_continuous = Event::HapticContinuous {
            time: 0.1,
            event_duration: 1.0,
            event_parameters: event_parameters.clone(),
        };

        let event_transient = Event::HapticTransient {
            time: 0.1,
            event_parameters,
        };

        let parameter_control_points_intensity = vec![
            ParameterCurveControlPoint {
                time: 0.6,
                parameter_value: 0.9,
            },
            ParameterCurveControlPoint {
                time: 0.7,
                parameter_value: 0.95,
            },
        ];
        let parameter_control_points_sharpness = vec![
            ParameterCurveControlPoint {
                time: 0.6,
                parameter_value: 0.9,
            },
            ParameterCurveControlPoint {
                time: 0.7,
                parameter_value: 0.95,
            },
        ];

        let parameter_curve_intensity = ParameterCurve {
            parameter_id: DynamicParameterId::HapticIntensityControl,
            time: 0.2,
            parameter_curve_control_points: parameter_control_points_intensity,
        };
        let parameter_curve_sharpness = ParameterCurve {
            parameter_id: DynamicParameterId::HapticSharpnessControl,
            time: 0.4,
            parameter_curve_control_points: parameter_control_points_sharpness,
        };

        let pattern = vec![
            Pattern::Event(event_continuous),
            Pattern::Event(event_transient),
            Pattern::ParameterCurve(parameter_curve_sharpness),
            Pattern::ParameterCurve(parameter_curve_intensity),
        ];

        let _ahap = Ahap {
            version: 1.0,
            metadata,
            pattern,
        };
    }

    fn load_file_from_test_data(path: &str) -> String {
        std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/test_data")
                .join(path),
        )
        .unwrap()
    }

    fn compare_v1_with_ahap(v1_path: &str, ahap_path: &str) {
        let v1_data: v1::DataModel =
            serde_json::from_str::<v1::DataModel>(&load_file_from_test_data(v1_path)).unwrap();
        let ahap_from_v1 = ios::v1::Ahap::from(v1_data);
        let ahap_reference_json = load_file_from_test_data(ahap_path);
        let ahap_reference_data = serde_json::from_str::<Ahap>(&ahap_reference_json).unwrap();
        assert_eq!(ahap_reference_data, ahap_from_v1);
    }

    ///Testing of deserializing an AHAP file with valid fields
    #[test]
    fn test_deserializing_valid_ahap() {
        let valid_ahap = load_file_from_test_data("ios/valid_fields.ahap");
        serde_json::from_str::<Ahap>(&valid_ahap).unwrap();
    }

    ///Testing of deserializing an AHAP file with invalid fields
    #[test]
    fn test_deserializing_invalid_ahap() {
        let valid_ahap = load_file_from_test_data("ios/invalid_fields.ahap");
        let err = serde_json::from_str::<Ahap>(&valid_ahap).unwrap_err();
        assert!(err
            .to_string()
            .contains("unknown variant `ParameterCurves`"));
    }

    ///Testing of deserializing an AHAP file exported from Studio Desktop
    #[test]
    fn test_deserializing_studio_export_ahap() {
        let studio_ahap = load_file_from_test_data("ios/studio_export.ahap");
        serde_json::from_str::<Ahap>(&studio_ahap).unwrap();
    }

    #[test]
    ///Testing of deserializing from AHAP file with required fields
    fn test_deserialize_ahap_required() {
        let required_ahap = load_file_from_test_data("ios/valid_required.ahap");
        serde_json::from_str::<Ahap>(&required_ahap).unwrap();
    }

    ///Testing conversion from v1 to AHAP with transients
    #[test]
    fn test_ahap_from_v1() {
        compare_v1_with_ahap("valid_v1.haptic", "ios/ahap_from_valid_v1.ahap");
    }

    ///Testing conversion from v1 to AHAP with required fields only (no emphasis)
    #[test]
    fn test_ahap_from_v1_required() {
        compare_v1_with_ahap(
            "valid_required_v1.haptic",
            "ios/ahap_from_valid_v1_required.ahap",
        );
    }

    #[test]
    fn test_ahap_from_v1_various_emphasis_count() {
        compare_v1_with_ahap(
            "ios/valid_v1_multiple_emphasis.haptic",
            "ios/valid_v1_multiple_transients.ahap",
        );
    }

    #[test]
    ///Testing v1 to AHAP conversion for various amount of points that could trigger corner cases
    fn test_ahap_from_v1_various_point_count() {
        // Make sure that clips with only 2 and 3 points get correctly converted to AHAP.
        // This tests PD-1292.
        compare_v1_with_ahap("ios/2_points.haptic", "ios/2_points.ahap");
        compare_v1_with_ahap("ios/3_points.haptic", "ios/3_points.ahap");

        // A ParameterCurve in AHAP can only contain up to 16 points. Verify that the chunking
        // algorithm deals correctly with the boundary condition.
        compare_v1_with_ahap("ios/16_points.haptic", "ios/16_points.ahap");
        compare_v1_with_ahap("ios/17_points.haptic", "ios/17_points.ahap");
    }

    #[test]
    ///Testing AHAP conversion of a clip that is longer than 30 seconds
    fn test_30_second_limit() {
        compare_v1_with_ahap("ios/long_clip.haptic", "ios/long_clip.ahap");
    }
}
