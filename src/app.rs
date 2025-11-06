use crate::profile_loader::{Browsers, Installation, Profile};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use serde::{Serialize, Deserialize};

pub struct CommandArguments {
    pub uri: String,
    pub browser_type: Browsers,
    pub executable: PathBuf,
    pub profile: Profile,
}

impl CommandArguments {
    pub fn create_command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        self.browser_type
            .add_args_to_command(&mut command, &self.profile.profile_path, &self.uri);
        #[cfg(target_os = "windows")]
        command.creation_flags(0x210);
        #[cfg(target_os = "linux")]
        command.process_group(0);
        command
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppCache {
    pub cfg_version: u8,
    pub installations: Vec<Installation>,
}

impl Default for AppCache {
    fn default() -> Self { Self { cfg_version: 0, installations: Vec::new() } }
}
