use directories::BaseDirs;
use ini_roundtrip as ini;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct ArgConstructor {
    uri_index: usize,
    profile_index: usize,
    args: Vec<String>,
}
impl ArgConstructor {
    fn construct_args<'a>(
        &'_ self,
        command: &'a mut Command,
        profile_path: &'_ Path,
        uri: &'_ str,
    ) -> &'a mut Command {
        let mut args: Vec<&OsStr> = self.args.iter().map(OsStr::new).collect();
        args[self.uri_index] = OsStr::new(uri);
        args[self.profile_index] = profile_path.as_os_str();
        command.args(args)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CustomBrowser {
    exe: String,
    name: String,
    args: ArgConstructor,
    ico_path: Option<PathBuf>, // Relative to installation path
    profile_ini: Option<PathBuf>,
    hard_profiles: Vec<Profile>, // Either one or none of the two can be empty / None
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Browsers {
    Firefox,
    FirefoxNightly,
    FirefoxBeta,
    FirefoxDeveloper,
    FirefoxPortable,
    Librewolf,
    LibrewolfPortable,
    Custom(CustomBrowser),
}
impl Browsers {
    /// Explores the directory to determine if it is of an implemented browser type
    ///
    /// # Errors
    ///
    /// This function returns an error whenever `std::fs::read_dir` would error, whether initially
    /// or while iterating over directory entries.
    pub fn detect_type(installation_path: &Path) -> Result<Option<Browsers>, std::io::Error> {
        let entries = std::fs::read_dir(installation_path)?;
        for entry in entries {
            let entry = entry?;
            // if !entry.file_type()?.is_file() {
            //     continue;
            // }
            #[cfg(target_os = "windows")]
            if entry.file_name() == "firefox.exe" {
                return Ok(Some(Self::differentiate_firefox_release(installation_path)));
            }
            #[cfg(target_os = "windows")]
            if entry.file_name() == "FirefoxPortable.exe" {
                return Ok(Some(Browsers::FirefoxPortable));
            }
            #[cfg(target_os = "windows")]
            if entry.file_name() == "librewolf.exe" {
                return Ok(Some(Browsers::Librewolf));
            }
            #[cfg(target_os = "windows")]
            if entry.file_name() == "LibreWolf-Portable.exe" {
                return Ok(Some(Browsers::LibrewolfPortable));
            }
        }
        Ok(None)
    }
    #[must_use]
    pub fn find_profiles(&self, installation_path: &Path) -> Vec<Profile> {
        let mut profiles: Vec<Profile> = Vec::new();
        match self {
            Browsers::Firefox
            | Browsers::FirefoxNightly
            | Browsers::FirefoxDeveloper
            | Browsers::FirefoxBeta =>
            {
                #[cfg(target_os = "windows")]
                if let Some(base_dirs) = BaseDirs::new() {
                    if let Ok(mut profile_ini) = parse_profiles_ini(
                        &base_dirs
                            .config_dir()
                            .join("Mozilla\\Firefox\\profiles.ini"),
                    ) {
                        profiles.append(&mut profile_ini);
                    }
                }
            }
            Browsers::FirefoxPortable => {
                #[cfg(target_os = "windows")]
                if installation_path.join("Data\\profile").exists() {
                    profiles.push(Profile {
                        name: "FirefoxPortable".parse().unwrap(),
                        profile_path: installation_path.join("Data\\profile"),
                    });
                }
                #[cfg(target_os = "windows")]
                if let Some(base_dirs) = BaseDirs::new() {
                    if let Ok(mut profile_ini) = parse_profiles_ini(
                        &base_dirs
                            .config_dir()
                            .join("Mozilla\\Firefox\\profiles.ini"),
                    ) {
                        profiles.append(&mut profile_ini);
                    }
                }
            }
            Browsers::Librewolf =>
            {
                #[cfg(target_os = "windows")]
                if let Some(base_dirs) = BaseDirs::new() {
                    if let Ok(mut profile_ini) =
                        parse_profiles_ini(&base_dirs.config_dir().join("librewolf\\profiles.ini"))
                    {
                        profiles.append(&mut profile_ini);
                    }
                }
            }
            Browsers::LibrewolfPortable => {
                #[cfg(target_os = "windows")]
                if installation_path.join("Profiles\\Default").exists() {
                    profiles.push(Profile {
                        name: "LibrewolfPortable".parse().unwrap(),
                        profile_path: installation_path.join("Profiles\\Default"),
                    });
                }
                #[cfg(target_os = "windows")]
                if let Some(base_dirs) = BaseDirs::new() {
                    if let Ok(mut profile_ini) =
                        parse_profiles_ini(&base_dirs.config_dir().join("librewolf\\profiles.ini"))
                    {
                        profiles.append(&mut profile_ini);
                    }
                }
            }
            Browsers::Custom(custom) => {
                if let Some(profile_ini_path) = &custom.profile_ini {
                    if let Ok(mut profile_ini) = parse_profiles_ini(profile_ini_path) {
                        profiles.append(&mut profile_ini);
                    }
                }
                profiles.append(&mut custom.hard_profiles.clone())
            }
        }
        profiles
    }
    #[must_use]
    pub fn get_name(&self) -> &str {
        match self {
            Browsers::Firefox => "Firefox",
            Browsers::FirefoxNightly => "Firefox Nightly",
            Browsers::FirefoxBeta => "Firefox Beta",
            Browsers::FirefoxDeveloper => "Firefox Developer",
            Browsers::FirefoxPortable => "Firefox Portable",
            Browsers::Librewolf => "Librewolf",
            Browsers::LibrewolfPortable => "Librewolf Portable",
            Browsers::Custom(custom) => &custom.name,
        }
    }
    #[must_use]
    pub fn get_exe_name(&self) -> &str {
        #[cfg(target_os = "windows")]
        match self {
            Browsers::Firefox
            | Browsers::FirefoxNightly
            | Browsers::FirefoxDeveloper
            | Browsers::FirefoxBeta => "firefox.exe",
            Browsers::FirefoxPortable => "FirefoxPortable.exe",
            Browsers::Librewolf => "librewolf.exe",
            Browsers::LibrewolfPortable => "LibrewolfPortable.exe",
            Browsers::Custom(custom) => &custom.exe,
        }
    }
    #[must_use]
    pub fn get_icon(&self, installation_path: &Path) -> Option<PathBuf> {
        match self {
            Browsers::Firefox
            | Browsers::FirefoxNightly
            | Browsers::FirefoxDeveloper
            | Browsers::FirefoxBeta
            | Browsers::Librewolf => {
                let ico = installation_path.join(r"browser\VisualElements\VisualElements_150.png");
                if ico.exists() {
                    Some(ico)
                } else {
                    None
                }
            }
            Browsers::LibrewolfPortable | Browsers::FirefoxPortable => {
                let ico = installation_path.join(
                    r"LibreWolf\browser\VisualElements\VisualElements_150
                .png",
                );
                if ico.exists() {
                    Some(ico)
                } else {
                    None
                }
            }
            Browsers::Custom(custom) => custom.ico_path.clone(),
        }
    }
    pub fn add_args_to_command<'a>(
        &self,
        c: &'a mut Command,
        profile_path: &Path,
        uri: &str,
    ) -> &'a mut Command {
        match self {
            Browsers::Firefox
            | Browsers::FirefoxNightly
            | Browsers::FirefoxBeta
            | Browsers::FirefoxDeveloper
            | Browsers::FirefoxPortable
            | Browsers::Librewolf
            | Browsers::LibrewolfPortable => {
                c.arg("--profile").arg(profile_path).arg("-url").arg(uri)
            }
            Browsers::Custom(custom) => custom.args.construct_args(c, profile_path, uri),
        }
    }
    #[cfg(target_os = "windows")]
    fn differentiate_firefox_release(installation_path: &Path) -> Browsers {
        let version = Command::new("powershell")
            .current_dir(installation_path)
            .arg("-Command")
            .arg(r"(Get-Item .\firefox.exe).VersionInfo.InternalName")
            .output()
            .unwrap(); // TODO error handling
        if let Ok(ver) = String::from_utf8(version.stderr) {
            match ver.as_str() {
                "Firefox Nightly" => Browsers::FirefoxNightly,
                "Firefox Beta" => Browsers::FirefoxBeta,
                "Firefox Developer" => Browsers::FirefoxDeveloper,
                _ => Browsers::Firefox,
            }
        } else {
            panic!("Wtf. Why did powershell give non-utf8.") // TODO handle more gracefully
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Installation {
    pub name: Option<String>,
    pub browser_type: Browsers,
    pub exe_path: PathBuf,
    pub symlinks: Vec<PathBuf>,
    pub preferred: HashMap<PathBuf, PathBuf>,
    pub profiles: Vec<Profile>,
    pub last_used: Option<Profile>,
}

impl Installation {
    /// Create an Installation from a path
    ///
    /// # Errors
    ///
    /// This function returns an error in any of the following situations:
    ///  - `std::fs::read_dir` returned an error, during initialisation or iterating over the path
    ///  - The path was not detected to be an implemented browser
    ///  - Could not find any profiles associated with the installation
    ///
    /// The latter two return custom `io::Error` with `ErrorKind::Other`
    pub fn from_installation_path(installation_path: &Path) -> Result<Self, std::io::Error> {
        let Some(type_) = Browsers::detect_type(installation_path)? else {
            return Err(std::io::Error::other("Unknown browser type"));
        };
        let profiles = type_.find_profiles(installation_path);
        if profiles.is_empty() {
            return Err(std::io::Error::other("No profiles found"));
        }
        Ok(Installation {
            name: None,
            browser_type: type_.clone(),
            exe_path: installation_path.join(type_.get_exe_name()),
            symlinks: Vec::new(),
            preferred: HashMap::default(),
            profiles,
            last_used: None,
        })
    }
    #[inline]
    #[must_use]
    pub fn get_icon(&self) -> Option<PathBuf> {
        self.browser_type.get_icon(&self.exe_path)
    }
    #[inline]
    #[must_use]
    pub fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or(self.browser_type.get_name())
    }
    /// Tries to add a symlink, returning `Ok(true)` if it is a symlink, otherwise `Ok(false)`
    ///
    /// # Errors
    ///
    /// If the path provided (or the installations `exe_path`) cannot be canonicalised
    #[allow(clippy::missing_panics_doc)]
    pub fn add_symlink(&mut self, installation_path: &Path) -> Result<bool, std::io::Error> {
        // .parent() won't ever return an error since self.exe_path is a join of a valid path and
        // the exe name of the browser, ergo self.exe_path should never be the root
        if self.exe_path.parent().unwrap().canonicalize()? == installation_path.canonicalize()? {
            self.symlinks.push(installation_path.to_path_buf());
            return Ok(true);
        }
        Ok(false)
    }
    pub fn remove_symlink(&mut self, installation_path: &Path) -> bool {
        if let Some(index) = self.symlinks.iter().position(|p| p == installation_path) {
            self.symlinks.remove(index);
            true
        } else {
            false
        }
    }
}

impl Display for Installation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Profile {
    name: String,
    pub profile_path: PathBuf,
}

impl Profile {
    #[must_use]
    pub fn is_open(&self) -> bool {
        let lockfile = self.profile_path.join("parent.lock");
        // Could also be false because of lacking permissions
        if !lockfile.exists() {
            return false;
        }
        // A locked file is a filesystem (OS) error and can't be explicitly matched upon
        // Presume other possible errors would cause the exists to fail first
        std::fs::File::open(lockfile).is_err()
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[allow(clippy::too_many_lines)]
fn parse_profiles_ini(profiles_ini: &Path) -> Result<Vec<Profile>, std::io::Error> {
    #[derive(PartialEq)]
    enum ParserState {
        InProfile,
        Other,
    }
    use ParserState::{InProfile, Other};

    let mut profiles: Vec<Profile> = Vec::new();

    let mut parser_state = Other;
    let mut name_buffer = String::new();
    let mut path_buffer = String::new();
    let mut isrelative_buffer: Option<bool> = None;

    for item in ini::Parser::new(&std::fs::read_to_string(profiles_ini)?) {
        match item {
            ini::Item::Section {
                name: section_name, ..
            } => {
                if section_name.starts_with("Profile") {
                    parser_state = InProfile;
                }
            }
            ini::Item::SectionEnd => {
                if parser_state == InProfile {
                    if name_buffer.is_empty() {
                        warn!(
                            "Missing `Name` key in a `Profile` section in the `profiles.ini` at {}",
                            profiles_ini.to_str().unwrap()
                        );
                        continue;
                    }
                    if path_buffer.is_empty() {
                        warn!(
                            "Missing `Path` key in a `Profile` section in the `profiles.ini` at {}",
                            profiles_ini.to_str().unwrap()
                        );
                        continue;
                    }
                    if isrelative_buffer.is_none() {
                        warn!(
                            "Missing `IsRelative` key in a `Profile` section in the `profiles.ini` at {}",
                            profiles_ini.to_str().unwrap()
                        );
                        continue;
                    }
                    profiles.push(Profile {
                        name: name_buffer,
                        profile_path: if isrelative_buffer.unwrap() {
                            profiles_ini.parent().unwrap().join(path_buffer)
                        } else {
                            path_buffer.parse().unwrap() // TODO Proper error handling 4 future me
                        },
                    });
                    name_buffer = String::new();
                    path_buffer = String::new();
                    isrelative_buffer = None;
                    parser_state = Other;
                };
            }
            ini::Item::Property {
                key: "Name",
                val: name_val,
                ..
            } => {
                if !name_buffer.is_empty() {
                    warn!(
                        "Multiple `Name` keys in a `Profile` section in the `profiles.ini` at {}",
                        profiles_ini.to_str().unwrap()
                    );
                    continue;
                }
                if let Some(val) = name_val {
                    name_buffer = String::from(val);
                } else {
                    warn!(
                        "Failed parsing `Name` value in a `Profile` section in the \
                    `profiles.ini` at {}",
                        profiles_ini.to_str().unwrap()
                    );
                }
            }
            ini::Item::Property {
                key: "Path",
                val: path_val,
                ..
            } => {
                if !path_buffer.is_empty() {
                    warn!(
                        "Multiple `Path` keys in a `Profile` section in the `profiles.ini` at {}",
                        profiles_ini.to_str().unwrap()
                    );
                    continue;
                }
                if let Some(val) = path_val {
                    path_buffer = String::from(val);
                } else {
                    warn!(
                        "Failed parsing `Path` value in a `Profile` section in the \
                    `profiles.ini` at {}",
                        profiles_ini.to_str().unwrap()
                    );
                }
            }
            ini::Item::Property {
                key: "IsRelative",
                val: isrelative_val,
                ..
            } => {
                if isrelative_buffer.is_some() {
                    warn!(
                        "Multiple `IsRelative` keys in a `Profile` section in the `profiles.ini` \
                        at {}",
                        profiles_ini.to_str().unwrap()
                    );
                    continue;
                }
                if isrelative_val.is_some() {
                    isrelative_buffer = isrelative_val.map(|num| num.trim() == "1");
                } else {
                    warn!(
                        "Failed parsing `IsRelative` value in a `Profile` section in the \
                    `profiles.ini` at {}",
                        profiles_ini.to_str().unwrap()
                    );
                }
            }
            _ => continue,
        }
    }
    Ok(profiles)
}
