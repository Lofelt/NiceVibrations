//! Crate containing Lofelt Data model related functions, schema and versioning.
pub mod emphasis;
pub mod interpolation;
pub mod ios;
pub mod test_utils;
pub mod v0;
pub mod v1;
pub mod version;
pub mod waveform;

pub use v1 as latest;
use version::*;

/// Constants for DataModel validation.
const MAX_ENVELOPE_AMPLITUDE: f32 = 1.0;
const MIN_ENVELOPE_AMPLITUDE: f32 = 0.0;

pub enum DataModel {
    V0(v0::DataModel),
    V1(v1::DataModel),
}

#[derive(PartialEq, Debug)]
pub enum VersionSupport {
    Full,
    Partial,
}

/// Receives a JSON string data with Lofelt Data and returns deserialized data with the correspondent
/// version of the Lofelt Data model.
pub fn from_json(data: &str) -> Result<DataModel, String> {
    match Version::from_json(data) {
        Version {
            major: 1,
            minor: _,
            patch: _,
        } => match serde_json::from_str::<v1::DataModel>(data) {
            Ok(deserialized_data) => match deserialized_data.validate() {
                // successfully deserialized
                Ok(validated_data) => Ok(DataModel::V1(validated_data)), // successfully validated datamodel
                Err(e) => Err(format!("Error validating V1: {}", e)),    // validation error
            },
            Err(e) => Err(format!("Error deserializing V1: {}", e)),
        },
        Version {
            major: 0,
            minor: 2,
            patch: 0,
        } => match serde_json::from_str::<v0::DataModel>(data) {
            Ok(deserialized_data) => match deserialized_data.validate() {
                // successfully deserialized
                Ok(validated_data) => Ok(DataModel::V0(validated_data)), // successfully validated datamodel
                Err(e) => Err(format!("Error validating V0: {}", e)),    // validation error
            },
            Err(e) => Err(format!("Error deserializing V0: {}", e)), // deserialization error
        },
        _ => Err(String::from("Unsupported version")),
    }
}

/// Like from_json(), but also upgrades the datamodel to the latest version.
pub fn latest_from_json(data: &str) -> Result<(VersionSupport, latest::DataModel), String> {
    upgrade_to_latest(&from_json(data)?)
}

/// Datamodel Validation trait
pub trait Validation {
    fn validate(self) -> Result<Self, String>
    where
        Self: Sized;
}

/// Upgrades Lofelt Data to the latest version available
pub fn upgrade_to_latest(data: &DataModel) -> Result<(VersionSupport, latest::DataModel), String> {
    match data {
        DataModel::V0(v0_data) => Ok((VersionSupport::Full, v1::DataModel::from(v0_data.clone()))),
        DataModel::V1(v1) => {
            if v1.version < latest::DataModel::CURRENT {
                // If the version of "data" is lower than CURRENT, we run upgrade code.
                // Example: CURRENT is 1.2, and the version of "data" is 1.1.
                let mut v1_latest = v1.clone();

                // TODO: Add upgrade code here, once we have a 1.x version
                v1_latest.version = latest::DataModel::CURRENT;

                Ok((VersionSupport::Full, v1_latest))
            } else if v1.version == latest::DataModel::CURRENT {
                Ok((VersionSupport::Full, v1.clone()))
            } else {
                // If the version of "data" is higher than CURRENT, we do nothing here.
                // Elsewhere a warning is printed.
                // This can happen when trying to load a .haptic file that was created
                // with a version of Studio Desktop that is more recent than the SDK.
                // Example: CURRENT is 1.3, and the version of "data" is 1.4.
                Ok((VersionSupport::Partial, v1.clone()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ios::v1::Ahap;
    use std::path::Path;

    fn load_file_from_test_data(path: &str) -> String {
        std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/test_data")
                .join(path),
        )
        .unwrap()
    }

    fn load_test_file_valid_v1() -> String {
        load_file_from_test_data("valid_v1.haptic")
    }

    #[test]
    fn test_valid_v1_from_json() {
        let data_json = load_test_file_valid_v1();

        match from_json(&data_json).unwrap() {
            DataModel::V1(data_v1) => assert_eq!(data_v1.version.major, 1),
            DataModel::V0(_) => panic!(),
        };
    }

    #[test]
    fn test_invalid_version_v1_from_json() {
        let data_json = load_file_from_test_data("invalid_version_v1.haptic");

        assert_eq!(
            from_json(&data_json).err(),
            Some("Unsupported version".to_string())
        );
    }

    #[test]
    ///Test if conversion from v1 to AHAP to string doesn't give any error
    fn test_v1_to_ahap_to_ahap_string() {
        let test_data = load_test_file_valid_v1();

        let data = from_json(&test_data).unwrap();

        match data {
            DataModel::V0(_) => panic!("Version should be V1"),

            DataModel::V1(v1_data) => {
                let ahap_data = Ahap::from(v1_data);
                ios::v1::Ahap::to_string(&ahap_data).unwrap();
            }
        }
    }

    /// Unit test for V0
    #[test]
    fn test_valid_v0_from_json() {
        let data_json = load_file_from_test_data("valid_v0.vij");

        match from_json(&data_json).unwrap() {
            DataModel::V1(_) => panic!("Should be a valid V0 file"),
            DataModel::V0(data_v0) => assert_eq!(data_v0.version.major, 0),
        };
    }

    /// Unit test for invalid V0 deserialization
    #[test]
    fn test_invalid_v0_from_json() {
        let data_json = load_file_from_test_data("no_voices_v0.vij");
        let err = from_json(&data_json).map(|_| ()).unwrap_err();
        assert!(
            err.contains("Error deserializing V0"),
            "Version should be Unreadable"
        );
    }

    /// Unit test for invalid V0 validation
    #[test]
    fn test_validation_v0_from_json() {
        let data_json = load_file_from_test_data("invalid_v0_conversions_transients.vij");
        let err = from_json(&data_json).map(|_| ()).unwrap_err();
        assert!(
            err.contains("Error validating V0"),
            "Validation should fail"
        );
    }

    ///Helper function to compare latest version of Lofelt haptic data with ahap data
    fn compare_latest_to_ahap(haptic_file_path: &str, ahap_file_path: &str) {
        let haptic_file_string = load_file_from_test_data(haptic_file_path);
        let ahap_file_string = load_file_from_test_data(ahap_file_path);

        //reference ahap data split to 2 ahaps
        let ahap_reference_data = serde_json::from_str::<ios::v1::Ahap>(&ahap_file_string).unwrap();
        let (ahap_reference_data_continuous, ahap_reference_data_transients) =
            ahap_reference_data.into_continuous_and_transients_ahaps();

        //lofelt haptic data upgraded to latest and split to 2 ahaps
        let haptic_data = from_json(&haptic_file_string).unwrap();
        let (_, latest) = upgrade_to_latest(&haptic_data).unwrap();

        let (ahap_data_continuous, ahap_data_transients) =
            ios::convert_to_transient_and_continuous_ahaps(latest);

        assert_eq!(ahap_reference_data_continuous, ahap_data_continuous);
        assert_eq!(ahap_reference_data_transients, ahap_data_transients);
    }

    #[test]
    ///Test conversion from v0 (VIJ from DSP) to AHAP
    fn test_v0_from_dsp_to_ahap() {
        compare_latest_to_ahap(
            "valid_v0_from_dsp.vij",
            "ios/ahap_from_valid_v0_from_dsp.ahap",
        );
    }

    ///Test conversion from v0 with transients to AHAP
    #[test]
    fn test_v0_to_ahap() {
        compare_latest_to_ahap("valid_v0.vij", "ios/ahap_from_valid_v0.ahap");
    }

    #[test]
    fn test_v1_to_ahap() {
        compare_latest_to_ahap(
            "ios/valid_v1_multiple_emphasis.haptic",
            "ios/valid_v1_multiple_transients.ahap",
        )
    }

    /// Unit test for invalid V1 validation
    #[test]
    fn test_validation_v1_from_json() {
        let data_json = load_file_from_test_data("validation_v1_amplitude.haptic");
        let err = from_json(&data_json).map(|_| ()).unwrap_err();
        assert!(
            err.contains("Error validating V1"),
            "Validation should fail"
        );
    }

    #[test]
    fn test_latest_from_json() {
        let data_json = load_file_from_test_data("valid_v0_conversion.vij");
        let target_data: v1::DataModel =
            serde_json::from_str(&load_file_from_test_data("valid_v1_from_v0.haptic")).unwrap();

        let (version_support, data_from_json) = latest_from_json(&data_json).unwrap();

        assert_eq!(data_from_json.signals, target_data.signals);
        assert_eq!(version_support, VersionSupport::Full);
    }

    // Unit test for loading .haptic file with a higher minor version than what we support
    #[test]
    #[cfg(not(target_os = "ios"))]
    fn test_load_newer_minor_version() {
        let data = load_file_from_test_data("v1_additional_fields.haptic");
        let (version_support, _) = latest_from_json(&data).unwrap();
        assert_eq!(version_support, VersionSupport::Partial);
    }

    // Unit test for default version when creating datamodel by hand
    #[test]
    fn test_default_version() {
        let data_v0 = v0::DataModel::default();
        let data_v1 = v1::DataModel::default();

        assert_eq!(
            data_v0.version,
            v0::DataModel::CURRENT,
            "Wrong version number: found {:?} expected {:?}",
            data_v0.version,
            v0::DataModel::CURRENT
        );
        assert_eq!(
            data_v1.version,
            v1::DataModel::CURRENT,
            "Wrong version number: found {:?} expected {:?}",
            data_v1.version,
            v1::DataModel::CURRENT
        );
    }
}
