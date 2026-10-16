//! The settings file, `config.toml`.
//!
//! keepclip writes it when the user picks the download folder, and reads it
//! on every run. People may edit it by hand.

use serde::Deserialize;
use std::fmt;
use std::path::PathBuf;

/// Format of the settings file, written as `version = 1` on its first line.
/// A change that would make an older file mean something else raises it, and
/// [`parse`] converts older files. A file without it counts as version 1.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    /// Where downloads are saved; `None` until the user picks a folder.
    pub download_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub enum ConfigError {
    Syntax(toml::de::Error),
    /// The file was written by a newer keepclip, in a format this one does
    /// not know.
    NewerVersion(u32),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Syntax(e) => write!(f, "{}", e.message()),
            ConfigError::NewerVersion(v) => write!(
                f,
                "the file has version = {v}, from a newer keepclip; this one reads up to \
                 version {FORMAT_VERSION}"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Deserialize)]
struct File {
    version: Option<u32>,
    download_dir: Option<String>,
}

pub fn parse(text: &str) -> Result<Config, ConfigError> {
    let file: File = toml::from_str(text).map_err(ConfigError::Syntax)?;
    let version = file.version.unwrap_or(1);
    if version > FORMAT_VERSION {
        return Err(ConfigError::NewerVersion(version));
    }
    Ok(Config {
        download_dir: file
            .download_dir
            .filter(|dir| !dir.trim().is_empty())
            .map(PathBuf::from),
    })
}

/// The whole file, with a comment for each setting.
pub fn render(config: &Config) -> String {
    let dir = config
        .download_dir
        .as_ref()
        .map_or(String::new(), |dir| dir.to_string_lossy().into_owned());
    format!(
        "# keepclip settings. You can edit this file; keepclip reads it on every run.\n\
         version = {FORMAT_VERSION}\n\
         \n\
         # Folder where downloads are saved. `keepclip --choose-folder` changes it.\n\
         download_dir = {}\n",
        quote(&dir)
    )
}

/// A TOML string. Windows paths read best as literal strings, between single
/// quotes, where a backslash is just a backslash.
fn quote(text: &str) -> String {
    if text.contains(['\'', '\n', '\r']) || text.chars().any(char::is_control) {
        toml::Value::String(text.to_string()).to_string()
    } else {
        format!("'{text}'")
    }
}
