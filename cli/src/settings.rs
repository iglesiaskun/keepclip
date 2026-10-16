//! The settings file, `%APPDATA%\keepclip\config.toml`, and the question
//! that fills it on the first run.

use crate::{Result, console};
use keepclip_core::config::{self, Config};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Where the settings file is: in `dir` when given, in the user's settings
/// folder otherwise.
pub fn file(dir: Option<&Path>) -> Result<PathBuf> {
    let dir = match dir {
        Some(dir) => dir.to_path_buf(),
        None => dirs::config_dir()
            .ok_or("Could not find the folder for settings.")?
            .join("keepclip"),
    };
    Ok(dir.join("config.toml"))
}

/// The settings, or the defaults when the file does not exist yet.
pub fn load(file: &Path) -> Result<Config> {
    match fs::read_to_string(file) {
        Ok(text) => config::parse(&text)
            .map_err(|e| format!("Could not read the settings in {}: {e}", file.display()).into()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(format!("Could not open {}: {e}", file.display()).into()),
    }
}

/// Asks for the download folder, creates it and saves it in the settings.
pub fn choose_folder(file: &Path, config: &mut Config) -> Result<PathBuf> {
    let suggestion = config
        .download_dir
        .clone()
        .or_else(dirs::download_dir)
        .or_else(dirs::home_dir)
        .ok_or("Could not find a folder to suggest.")?;
    println!("\nWhere should keepclip save the downloads?");
    let folder = loop {
        let typed = console::ask(&format!(
            "Press Enter for {}, or type another folder: ",
            suggestion.display()
        ))?;
        let folder = if typed.is_empty() {
            suggestion.clone()
        } else {
            PathBuf::from(expand_variables(typed.trim_matches('"'), |name| {
                std::env::var(name).ok()
            }))
        };
        if !folder.is_absolute() {
            println!("Type the whole path, such as C:\\Users\\you\\Videos.");
            continue;
        }
        match fs::create_dir_all(&folder) {
            Ok(()) => break folder,
            Err(e) => println!("Could not use {}: {e}", folder.display()),
        }
    };
    config.download_dir = Some(folder.clone());
    save(file, config)?;
    println!("Saved in {}.", file.display());
    Ok(folder)
}

/// Writes the settings through a temporary file, so a crash never leaves
/// half a file behind.
fn save(file: &Path, config: &Config) -> Result<()> {
    let fail = |e: std::io::Error| format!("Could not save {}: {e}", file.display());
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(fail)?;
    }
    let temporary = file.with_extension("toml.tmp");
    fs::write(&temporary, config::render(config)).map_err(fail)?;
    fs::rename(&temporary, file).map_err(fail)?;
    Ok(())
}

/// Replaces `%NAME%` with the value of the environment variable, as Windows
/// does. Unknown names stay as they are.
fn expand_variables(text: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('%') else { break };
        let name = &after[..end];
        out.push_str(&rest[..start]);
        match lookup(name).filter(|_| !name.is_empty()) {
            Some(value) => {
                out.push_str(&value);
                rest = &after[end + 1..];
            }
            None => {
                // Keep the first % and look for a name from the second one.
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(name: &str) -> Option<String> {
        (name == "USERPROFILE").then(|| "C:\\Users\\you".to_string())
    }

    #[test]
    fn expands_known_variables() {
        assert_eq!(
            expand_variables("%USERPROFILE%\\Videos", lookup),
            "C:\\Users\\you\\Videos"
        );
        assert_eq!(expand_variables("%NOPE%\\x", lookup), "%NOPE%\\x");
        assert_eq!(
            expand_variables("100% %USERPROFILE%", lookup),
            "100% C:\\Users\\you"
        );
        assert_eq!(expand_variables("50%", lookup), "50%");
    }
}
