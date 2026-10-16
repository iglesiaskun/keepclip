//! The settings file.

use keepclip_core::config::{self, Config, ConfigError, FORMAT_VERSION};
use std::path::PathBuf;

#[test]
fn a_written_file_reads_back_the_same() {
    for dir in [
        "C:\\Users\\you\\Videos",
        "D:\\Vídeos\\keepclip 🎬",
        "C:\\Users\\O'Brien\\Videos",
        "\\\\server\\share\\clips",
    ] {
        let config = Config {
            download_dir: Some(PathBuf::from(dir)),
        };
        let text = config::render(&config);
        assert_eq!(config::parse(&text).unwrap(), config, "{text}");
    }
}

#[test]
fn the_file_starts_with_its_version_and_explains_itself() {
    let text = config::render(&Config {
        download_dir: Some(PathBuf::from("C:\\Users\\you\\Videos")),
    });
    let first_setting = text.lines().find(|l| !l.starts_with('#')).unwrap();
    assert_eq!(first_setting, format!("version = {FORMAT_VERSION}"));
    // Windows paths go between single quotes, where backslashes stay as they are.
    assert!(text.contains("download_dir = 'C:\\Users\\you\\Videos'"));
    assert!(text.contains("# Folder where downloads are saved."));
}

#[test]
fn a_file_without_version_or_folder_is_fine() {
    assert_eq!(config::parse("").unwrap(), Config::default());
    assert_eq!(
        config::parse("download_dir = \"\"").unwrap(),
        Config::default()
    );
    assert_eq!(
        config::parse("download_dir = 'E:\\Clips'\nfuture_setting = true")
            .unwrap()
            .download_dir,
        Some(PathBuf::from("E:\\Clips"))
    );
}

#[test]
fn a_newer_format_is_refused() {
    let error = config::parse("version = 99").unwrap_err();
    assert!(matches!(error, ConfigError::NewerVersion(99)));
    assert!(error.to_string().contains("newer keepclip"));
}

#[test]
fn broken_toml_is_an_error() {
    assert!(matches!(
        config::parse("download_dir = 'unclosed"),
        Err(ConfigError::Syntax(_))
    ));
}
