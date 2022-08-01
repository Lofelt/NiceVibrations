//! Version model module
//!
//! The model version follows the ideas of [Semantic Versioning][1]:
//!
//! [1]: https://semver.org/

/// A trait that is implemented by each haptic data model to provide its current version
use serde::{Deserialize, Serialize};

///Lofelt data models versioning structure
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
pub struct Version {
    pub major: u32,
    #[serde(default = "Version::default_minor_patch")]
    pub minor: u32,
    #[serde(default = "Version::default_minor_patch")]
    pub patch: u32,
}

impl Version {
    ///Default value 0 to minor and patch, when only major is deserialized
    fn default_minor_patch() -> u32 {
        0
    }

    pub fn from_json(data: &str) -> Version {
        #[derive(Deserialize)]
        /// Helper struct to deserialize the version without needing the full DataModel
        pub struct VersionCheck {
            pub version: Version,
        }

        match serde_json::from_str::<VersionCheck>(data) {
            Ok(checker) => checker.version,
            Err(_) => Version::default(),
        }
    }
}

/// Default version structure values
impl Default for Version {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 2,
            patch: 0,
        }
    }
}

pub trait DataModelVersion {
    /// The latest revision of the model
    const CURRENT: Version;

    /// The revision of the model instance
    fn version(&self) -> &Version;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::eq_op)]
    fn version_cmp() {
        assert!(
            Version {
                major: 2,
                minor: 0,
                patch: 0,
            } > Version {
                major: 1,
                minor: 0,
                patch: 0,
            }
        );
        assert!(
            Version {
                major: 2,
                minor: 1,
                patch: 0,
            } > Version {
                major: 2,
                minor: 0,
                patch: 0,
            }
        );
        assert!(
            Version {
                major: 2,
                minor: 2,
                patch: 0,
            } == Version {
                major: 2,
                minor: 2,
                patch: 0,
            }
        );
        assert!(
            Version {
                major: 1,
                minor: 2,
                patch: 1,
            } < Version {
                major: 1,
                minor: 2,
                patch: 2,
            }
        );
    }
}
