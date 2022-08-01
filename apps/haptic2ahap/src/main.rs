//! This application allows the user to convert a Lofelt .haptic file to an Apple .ahap file, using lofelt-sdk/core/datamodel crate functions.
//!
//! This should be used internally in Lofelt only.

use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use datamodel::ios::v1::Ahap;
use std::{fs::File, io::Write, path::Path};

fn main() -> Result<(), String> {
    let matches = App::new("haptic2ahap")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("INPUT")
                .help("Input .haptic file to be converted to .ahap")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("NO_SPLIT")
                .long("no-split")
                .short("n")
                .help("Create a unified AHAP file instead of splitting it into two (continuous and transients). \
                       Note that such a unified AHAP file will not play back transients correctly, as the \
                       CHParameterCurves for the CHContinuousEvents will also be applied to CHTransients, \
                       thereby undesirably modifying the intensity and sharpness of the transients.\n\
                       For correct playback, the two split AHAPs should be played in parallel."),
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    let input_file = matches.value_of("INPUT").unwrap();
    let input_filename = input_file.strip_suffix(".haptic");
    let split = !matches.is_present("NO_SPLIT");

    //try load haptic file if file has .haptic extension
    match input_filename {
        Some(filename) => {
            let haptic_data = load_haptic_data_from_file(input_file)?;

            if split {
                let ahap_data =
                    datamodel::ios::convert_to_transient_and_continuous_ahaps(haptic_data);

                export_string_to_ahap_file(
                    &[filename, "_continuous"].concat(),
                    &datamodel::ios::v1::Ahap::to_string_pretty(&ahap_data.0)?,
                )?;

                if ahap_data.1.as_ref().is_some() {
                    export_string_to_ahap_file(
                        &[filename, "_transients"].concat(),
                        &datamodel::ios::v1::Ahap::to_string_pretty(&ahap_data.1.unwrap())?,
                    )?;
                }
            } else {
                let ahap = Ahap::from(haptic_data);
                export_string_to_ahap_file(
                    &[filename, ""].concat(),
                    &datamodel::ios::v1::Ahap::to_string_pretty(&ahap)?,
                )?;
            }
        }
        None => return Err(format!("Input '{}' should be a .haptic file", input_file)),
    }

    Ok(())
}

/// Loads latest  haptic data from file
/// - path: File path to load haptic data from
fn load_haptic_data_from_file(path: &str) -> Result<datamodel::latest::DataModel, String> {
    let path = std::fs::canonicalize(path)
        .map_err(|err| format!("Error reading input from '{:?}': {}", path, err))?;
    let haptic_json_string = std::fs::read_to_string(&path)
        .map_err(|err| format!("Error reading input from '{:?}': {}", path, err))?;
    let data_model = datamodel::from_json(&haptic_json_string)?;
    let (_, data_model) = datamodel::upgrade_to_latest(&data_model)?;
    Ok(data_model)
}

///Exports a string to `filename`.ahap file
/// - filename: name of ahap file
/// - data: String slice which contains data to be exported to file
fn export_string_to_ahap_file(filename: &str, data: &str) -> Result<(), String> {
    let output_file = format!("{}.ahap", filename);
    let path = Path::new(&output_file);
    let display = path.display();

    // Open a file in write-only mode
    match File::create(&path) {
        Err(e) => Err(format!("Couldn't create {}: {}", display, e)),
        Ok(mut file) => {
            // Write the ahap data string to `file`
            match file.write_all(data.as_bytes()) {
                Err(e) => Err(format!("Couldn't write to {}: {}", display, e)),
                Ok(_) => Ok(()),
            }
        }
    }
}
