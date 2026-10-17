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

impl Args {
    /// Nothing to do was given: keepclip was opened like any app, from the
    /// Start menu or a shortcut, perhaps with `--dir`.
    pub fn nothing_given(&self) -> bool {
        self.link.is_none() && !self.clipboard && !self.choose_folder
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Run(Args),
    Help,
    Version,
}

/// The help, for the program called `program`. `without_link` says, in two
/// short lines, what it does when given no link.
pub fn help(program: &str, without_link: [&str; 2]) -> String {
    let [first, second] = without_link;
    format!(
        "\
Download a video from a copied link.

Usage:
  {program} [LINK]            downloads LINK; {first}
  {pad}                   {second}
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
    fn nothing_given_is_opening_the_app() {
        assert!(run(&[]).unwrap().nothing_given());
        assert!(run(&["--dir", "C:\\test"]).unwrap().nothing_given());
        for words in [
            &["--clipboard"][..],
            &["--choose-folder"],
            &["https://example.com/v/1"],
        ] {
            assert!(!run(words).unwrap().nothing_given(), "{words:?}");
        }
    }

    #[test]
    fn the_help_names_the_program() {
        let text = help("keepclip-cli", ["without it, the link in the", "clipboard"]);
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
