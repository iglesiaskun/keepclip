//! The command line, the same for the window and the console.

use std::path::PathBuf;

/// What the command line asks for.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Args {
    pub link: Option<String>,
    pub clipboard: bool,
    pub choose_folder: bool,
    /// Folder of the settings file, instead of `%APPDATA%\keepclip`.
    pub dir: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Run(Args),
    Help,
    Version,
}

/// The help, for the program called `program`.
pub fn help(program: &str) -> String {
    format!(
        "\
Download a video from a copied link.

Usage:
  {program} [LINK]            downloads LINK; without it, the link in the
  {pad}                   clipboard, or one you paste
  {program} --clipboard       downloads the link in the clipboard
  {program} --choose-folder   picks the download folder again

Options:
  --dir FOLDER   keeps the settings file in FOLDER instead of
                 %APPDATA%\\keepclip (handy for testing)
  --help         shows this help
  --version      shows the version
",
        pad = " ".repeat(program.len())
    )
}

/// Reads the words after the program name.
pub fn parse(words: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut words = words.into_iter();
    let mut args = Args::default();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--help" | "-h" | "/?" => return Ok(Command::Help),
            "--version" | "-V" => return Ok(Command::Version),
            "--clipboard" => args.clipboard = true,
            "--choose-folder" => args.choose_folder = true,
            "--dir" => {
                let dir = words.next().ok_or("--dir needs a folder")?;
                args.dir = Some(PathBuf::from(dir));
            }
            _ if word.starts_with('-') => return Err(format!("unknown option {word}")),
            _ if args.link.is_some() => return Err("give only one link".to_string()),
            _ => args.link = Some(word),
        }
    }
    if args.clipboard && args.link.is_some() {
        return Err("give a link or --clipboard, not both".to_string());
    }
    Ok(Command::Run(args))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(words: &[&str]) -> Result<Args, String> {
        match parse(words.iter().map(|w| w.to_string()))? {
            Command::Run(args) => Ok(args),
            other => Err(format!("{other:?}")),
        }
    }

    #[test]
    fn takes_a_link_or_the_clipboard() {
        let args = run(&["https://example.com/v/1"]).unwrap();
        assert_eq!(args.link.as_deref(), Some("https://example.com/v/1"));
        assert!(run(&["--clipboard"]).unwrap().clipboard);
        assert!(run(&["--clipboard", "https://example.com/v/1"]).is_err());
        assert!(run(&["https://a.example", "https://b.example"]).is_err());
    }

    #[test]
    fn reads_the_settings_folder() {
        let args = run(&["--dir", "C:\\test", "--choose-folder"]).unwrap();
        assert_eq!(args.dir, Some(PathBuf::from("C:\\test")));
        assert!(args.choose_folder);
        assert!(run(&["--dir"]).is_err());
        assert!(run(&["--frobnicate"]).is_err());
    }

    #[test]
    fn help_and_version_end_the_reading() {
        let parse = |w: &[&str]| parse(w.iter().map(|w| w.to_string()));
        assert_eq!(parse(&["--help", "--frobnicate"]), Ok(Command::Help));
        assert_eq!(parse(&["-V"]), Ok(Command::Version));
    }

    #[test]
    fn the_help_names_the_program() {
        let text = help("keepclip-cli");
        assert!(text.contains("  keepclip-cli --clipboard  "));
        // The second line of a usage lines up with the first.
        let lines: Vec<&str> = text.lines().collect();
        let first = lines.iter().position(|l| l.contains("[LINK]")).unwrap();
        assert_eq!(
            lines[first].find("downloads"),
            lines[first + 1].find("clipboard")
        );
    }
}
