#[cfg(feature = "CLI")]
mod console;
#[cfg(feature = "GUI")]
mod gui;

use crate::profile_loader::Installation;
use std::process::Command;

#[cfg(all(feature = "CLI", not(feature = "GUI")))]
#[must_use]
pub fn open_dialog(uri: String, installations: &mut Vec<Installation>) -> Option<Command> {
    Some(console::open_dialog(uri, installations)?.create_command())
}

#[cfg(all(feature = "GUI", not(feature = "CLI")))]
#[must_use]
pub fn open_dialog(uri: String, installations: &mut Vec<Installation>) -> Option<Command> {
    Some(gui::open_dialog(uri, installations)?.create_command())
}

#[cfg(any(
    all(feature = "CLI", feature = "GUI"),
    all(not(feature = "CLI"), not(feature = "GUI"))
))]
pub fn open_dialog(uri: &str, installations: &mut Vec<Installation>) -> Command {
    compile_error!("You need to select either GUI or CLI!")
}

#[cfg(all(feature = "CLI", not(feature = "GUI")))]
pub fn open_config(installations: &mut Vec<Installation>) {
    todo!()
}

#[cfg(all(feature = "GUI", not(feature = "CLI")))]
pub fn open_config(installations: &mut Vec<Installation>) {
    todo!()
}

#[cfg(any(
    all(feature = "CLI", feature = "GUI"),
    all(not(feature = "CLI"), not(feature = "GUI"))
))]
pub fn open_config(installations: &mut Vec<Installation>) {
    compile_error!("You need to select either GUI or CLI!")
}
