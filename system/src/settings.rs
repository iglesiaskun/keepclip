//! The settings file, `%APPDATA%\keepclip\config.toml`.

use crate::Result;
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

/// The folder to suggest for downloads: the one in use, or the user's
/// Downloads folder.
pub fn suggested_folder(config: &Config) -> Result<PathBuf> {
    Ok(config
        .download_dir
        .clone()
        .or_else(dirs::download_dir)
        .or_else(dirs::home_dir)
        .ok_or("Could not find a folder to suggest.")?)
}

/// Creates `folder` and saves it as the download folder.
pub fn use_folder(file: &Path, config: &mut Config, folder: &Path) -> Result<()> {
    fs::create_dir_all(folder).map_err(|e| format!("Could not use {}: {e}", folder.display()))?;
    config.download_dir = Some(folder.to_path_buf());
    save(file, config)
}

/// Writes the settings through a temporary file, so a crash never leaves
/// half a file behind.
pub fn save(file: &Path, config: &Config) -> Result<()> {
    let fail = |e: std::io::Error| format!("Could not save {}: {e}", file.display());
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(fail)?;
    }
    let temporary = file.with_extension("toml.tmp");
    fs::write(&temporary, config::render(config)).map_err(fail)?;
    fs::rename(&temporary, file).map_err(fail)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chosen_folder_is_created_and_saved() {
        let root =
            std::env::temp_dir().join(format!("keepclip-test-settings-{}", std::process::id()));
        let settings = root.join("settings").join("config.toml");
        let folder = root.join("videos");
        let mut config = Config::default();
        use_folder(&settings, &mut config, &folder).unwrap();
        assert!(folder.is_dir());
        assert_eq!(load(&settings).unwrap().download_dir, Some(folder.clone()));
        assert_eq!(suggested_folder(&config).unwrap(), folder);
        let _ = fs::remove_dir_all(&root);
    }
}
