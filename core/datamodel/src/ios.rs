//! Contains iOS data models
pub mod v1;
use crate::*;

use v1::*;

///Converts latest version of Lofelt Data to two AHAPs with continuous and transients events
pub fn convert_to_transient_and_continuous_ahaps(data: latest::DataModel) -> (Ahap, Option<Ahap>) {
    Ahap::from(data).into_continuous_and_transients_ahaps()
}
