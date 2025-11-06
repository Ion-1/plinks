use crate::app::CommandArguments;
use crate::profile_loader::{Installation, Profile};
use inquire::error::InquireResult;
use inquire::formatter::OptionFormatter;
use inquire::list_option::ListOption;
use inquire::{Confirm, InquireError, Select};
use log::{debug, error, info, trace, warn};
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::path::{Path, PathBuf};

enum Choice<T: Display> {
    LastUsed,
    Option(T),
    Back,
}
impl<T: Display> Display for Choice<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Choice::LastUsed => {
                write!(f, "Last Used")
            }
            Choice::Option(object) => {
                write!(f, "{object}")
            }
            Choice::Back => {
                write!(f, "Back")
            }
        }
    }
}

pub struct DebugDisplay<T>(T);
impl<T: std::fmt::Debug> Display for DebugDisplay<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

pub struct CommandArgsBuilder<'a> {
    pub installations: &'a mut Vec<Installation>,
    pub uri: String,
    pub selected_installation_idx: Option<usize>,
    pub selected_profile: Option<Profile>,
    pub selected_exe_path: Option<PathBuf>,
}
impl CommandArgsBuilder<'_> {
    pub fn into_commandargs(mut self) -> Option<CommandArguments> {
        if self.selected_installation_idx.is_none()
            | self.selected_profile.is_none()
            | self.selected_exe_path.is_none()
        {
            return None;
        }
        Some(CommandArguments {
            browser_type: self.selected_installation().unwrap().browser_type.clone(),
            uri: self.uri,
            executable: self.selected_exe_path.unwrap(),
            profile: self.selected_profile.unwrap(),
        })
    }
    pub fn selected_installation(&mut self) -> Option<&mut Installation> {
        self.selected_installation_idx
            .as_mut()
            .map(|i| self.installations.get_mut(*i).unwrap())
    }
}

fn unpack_inquireresult<T>(res: InquireResult<T>) -> Option<T> {
    match res {
        Ok(val) => Some(val),
        Err(InquireError::OperationCanceled) => {
            info!("Dialog canceled by user");
            None
        }
        Err(InquireError::NotTTY) => {
            error!("Input is not TTY, required for inquire");
            None
        }
        Err(err) => {
            error!("Shit happened. {}", err);
            None
        }
    }
}

fn prompt_for_installation(mut builder: Box<CommandArgsBuilder>) -> Option<CommandArguments> {
    let ans: InquireResult<ListOption<String>> = Select::new(
        format!(
            "URI: {}\nWhich installation would you like to open it with?",
            builder.uri
        )
        .as_str(),
        builder
            .installations
            .iter()
            .map(|i| i.get_name().to_string())
            .collect(),
    )
    .raw_prompt();
    let ans = unpack_inquireresult(ans)?;
    builder.selected_installation_idx = Some(ans.index);
    prompt_for_profile(builder)
}

fn prompt_for_profile(mut builder: Box<CommandArgsBuilder>) -> Option<CommandArguments> {
    let ans: InquireResult<Choice<&Profile>> = Select::new(
        "Which profile would you like to use?",
        builder
            .selected_installation()
            .expect("Prompting for profile without a selected installation!")
            .profiles
            .iter()
            .map(Choice::Option)
            .chain(std::iter::once(Choice::Back))
            .collect(),
    )
    .prompt();
    let ans = unpack_inquireresult(ans)?;
    match ans {
        Choice::Back => prompt_for_installation(builder),
        Choice::Option(profile) => {
            builder.selected_profile = Some(profile.clone());
            prompt_for_exe_path(builder)
        }
        Choice::LastUsed => unreachable!(),
    }
}

fn prompt_for_exe_path(mut builder: Box<CommandArgsBuilder>) -> Option<CommandArguments> {
    let installation = builder
        .installations
        .get_mut(
            builder
                .selected_installation_idx
                .expect("Prompting for exe path without a selected installation!"),
        )
        .unwrap();
    let profile = builder
        .selected_profile
        .as_ref()
        .expect("Prompting for exe path without a selected profile!");
    let preferred = installation.preferred.get(&profile.profile_path);
    let ans: InquireResult<Choice<DebugDisplay<&PathBuf>>> = Select::new(
        "With which executable would you like to open it with?",
        preferred
            .into_iter()
            .map(|_| Choice::LastUsed)
            .chain(
                std::iter::once(&installation.exe_path)
                    .map(DebugDisplay)
                    .map(Choice::Option),
            )
            .chain(
                installation
                    .symlinks
                    .iter()
                    .map(DebugDisplay)
                    .map(Choice::Option)
                    .chain(std::iter::once(Choice::Back)),
            )
            .collect(),
    )
    .prompt();
    let ans = unpack_inquireresult(ans)?;
    match ans {
        Choice::Back => prompt_for_profile(builder),
        Choice::LastUsed => {
            builder.selected_exe_path = Some(
                installation
                    .preferred
                    .get(&profile.profile_path)
                    .unwrap()
                    .clone(),
            );
            Some(
                builder
                    .into_commandargs()
                    .expect("Invalid state in CommandArgsBuilder encountered"),
            )
        }
        Choice::Option(val) => {
            builder.selected_exe_path = Some(val.0.clone());
            installation
                .preferred
                .insert(profile.profile_path.clone(), val.0.to_path_buf());
            Some(
                builder
                    .into_commandargs()
                    .expect("Invalid state in CommandArgsBuilder encountered"),
            )
        }
    }
}

pub fn open_dialog(uri: String, installations: &mut Vec<Installation>) -> Option<CommandArguments> {
    // Need to Box to use explicit tail calls (PassMode::Indirect unsupported)
    // Using become seems to break inquire
    prompt_for_installation(Box::from(CommandArgsBuilder {
        uri,
        installations,
        selected_installation_idx: None,
        selected_profile: None,
        selected_exe_path: None,
    }))
}
