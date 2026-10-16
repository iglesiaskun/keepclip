//! keepclip: downloads a video from a copied link. It asks yt-dlp what the
//! link offers, shows a menu with the size of each choice, downloads the one
//! picked into the download folder and says where the file is.

mod clipboard;
mod console;
mod fetch;
mod notify;
mod settings;
mod tools;
mod ytdlp;

use keepclip_core::download::Request;
use keepclip_core::info::VideoInfo;
use keepclip_core::menu::{self, Choice};
use keepclip_core::progress::{self, Event, Status};
use keepclip_core::{link, units};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

/// Errors end the run with a message; there is nothing to recover.
type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

const HELP: &str = "\
Download a video from a copied link.

Usage:
  keepclip [LINK]            downloads LINK; without it, the link in the
                             clipboard, or one you paste
  keepclip --clipboard       downloads the link in the clipboard
  keepclip --choose-folder   picks the download folder again

Options:
  --dir FOLDER   keeps the settings file in FOLDER instead of
                 %APPDATA%\\keepclip (handy for testing)
  --help         shows this help
  --version      shows the version
";

/// What the command line asks for.
#[derive(Debug, Default)]
struct Args {
    link: Option<String>,
    clipboard: bool,
    choose_folder: bool,
    dir: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(Some(args)) => args,
        Ok(None) => return ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("keepclip: {e}\n\n{HELP}");
            return ExitCode::FAILURE;
        }
    };
    println!("keepclip {}", env!("CARGO_PKG_VERSION"));
    match run(&args) {
        Ok(Finish::Saved { notified }) => {
            if console::is_own_window() {
                if notified {
                    // The notification stays in the notification center;
                    // a moment is enough to read the console.
                    std::thread::sleep(Duration::from_secs(2));
                } else {
                    console::wait_for_enter("Press Enter to close.");
                }
            }
            ExitCode::SUCCESS
        }
        Ok(Finish::Nothing) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\n{e}");
            if console::is_own_window() {
                console::wait_for_enter("Press Enter to close.");
            }
            ExitCode::FAILURE
        }
    }
}

/// `Ok(None)` when the command line was fully handled (`--help`).
fn parse_args(mut words: impl Iterator<Item = String>) -> Result<Option<Args>, String> {
    let mut args = Args::default();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--help" | "-h" | "/?" => {
                print!("{HELP}");
                return Ok(None);
            }
            "--version" | "-V" => {
                println!("keepclip {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
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
    Ok(Some(args))
}

enum Finish {
    Saved {
        notified: bool,
    },
    /// Cancelled, or only the folder was changed.
    Nothing,
}

fn run(args: &Args) -> Result<Finish> {
    let settings_file = settings::file(args.dir.as_deref())?;
    let mut config = settings::load(&settings_file)?;
    if args.choose_folder {
        settings::choose_folder(&settings_file, &mut config)?;
        if args.link.is_none() && !args.clipboard {
            return Ok(Finish::Nothing);
        }
    }

    let Some(url) = find_link(args)? else {
        return Ok(Finish::Nothing);
    };
    println!("Link: {url}");

    let folder = match &config.download_dir {
        Some(folder) => folder.clone(),
        None => settings::choose_folder(&settings_file, &mut config)?,
    };

    let ytdlp = tools::ready()?;
    println!("Reading the link...");
    let info = ytdlp.info(&url)?;
    let Some((video, playlist_item)) = pick_video(&info)? else {
        return Ok(Finish::Nothing);
    };

    let choices = menu::build(video);
    if choices.is_empty() {
        return Err("yt-dlp found nothing to download at this link.".into());
    }
    describe(video);
    let Some(choice) = pick_choice(&choices)? else {
        return Ok(Finish::Nothing);
    };

    std::fs::create_dir_all(&folder)
        .map_err(|e| format!("Could not create the folder {}: {e}", folder.display()))?;
    println!("\nSaving to {}", folder.display());
    let request = Request {
        url: &url,
        choice,
        folder: &folder,
        playlist_item,
    };
    let saved = download(&ytdlp, &request)?;
    let saved = Path::new(&saved);
    println!("\nSaved: {}", saved.display());
    let notified = notify::saved(saved);
    Ok(Finish::Saved { notified })
}

/// The link from the command line, the clipboard or the keyboard.
fn find_link(args: &Args) -> Result<Option<String>> {
    if let Some(given) = &args.link {
        return match link::find(given) {
            Some(url) => Ok(Some(url.to_string())),
            None => Err(format!("\"{given}\" is not a web link (http:// or https://).").into()),
        };
    }
    let copied = clipboard::text();
    if let Some(url) = copied.as_deref().and_then(link::find) {
        return Ok(Some(url.to_string()));
    }
    if args.clipboard {
        return Err("The clipboard has no link. Copy the link of a video and try again.".into());
    }
    loop {
        let typed = console::ask("Paste a link (Enter alone quits): ")?;
        if typed.is_empty() {
            return Ok(None);
        }
        match link::find(&typed) {
            Some(url) => return Ok(Some(url.to_string())),
            None => println!("That is not a web link (http:// or https://)."),
        }
    }
}

/// The video to download, and its position when it is part of a playlist.
fn pick_video(info: &VideoInfo) -> Result<Option<(&VideoInfo, Option<u64>)>> {
    if !info.is_playlist() {
        return Ok(Some((info, None)));
    }
    let videos: Vec<&VideoInfo> = info.videos().collect();
    match videos.as_slice() {
        [] => Err("This link has no video that can be downloaded.".into()),
        [only] => Ok(Some((only, only.playlist_index))),
        _ => {
            println!(
                "\nThis link has {} videos{}:",
                videos.len(),
                info.title
                    .as_deref()
                    .map_or(String::new(), |t| format!(" ({t})"))
            );
            for (n, video) in videos.iter().enumerate() {
                let title = video.title.as_deref().unwrap_or(&video.id);
                println!("  {:>2}) {title}", n + 1);
            }
            let Some(n) = console::pick_number("Which one", videos.len())? else {
                return Ok(None);
            };
            let video = videos[n - 1];
            // Without its position, yt-dlp would download the whole list.
            let position = video.playlist_index.unwrap_or(n as u64);
            Ok(Some((video, Some(position))))
        }
    }
}

fn describe(video: &VideoInfo) {
    let title = video.title.as_deref().unwrap_or(&video.id);
    let mut line = format!("\n{title}");
    if let Some(uploader) = &video.uploader {
        line.push_str(&format!(" | {uploader}"));
    }
    if let Some(seconds) = video.duration {
        line.push_str(&format!(" | {}", units::duration(seconds as u64)));
    }
    println!("{line}");
}

fn pick_choice(choices: &[Choice]) -> Result<Option<&Choice>> {
    println!();
    let width = choices.iter().map(|c| c.label().len()).max().unwrap_or(0);
    for (n, choice) in choices.iter().enumerate() {
        println!(
            "  {:>2}) {:<width$}  {}",
            n + 1,
            choice.label(),
            choice.size_text()
        );
    }
    Ok(console::pick_number("Choose", choices.len())?.map(|n| &choices[n - 1]))
}

/// Downloads with a progress line and returns the path of the saved file.
fn download(ytdlp: &ytdlp::YtDlp, request: &Request<'_>) -> Result<String> {
    let mut line = console::StatusLine::default();
    let mut saved = None;
    let mut downloaded = false;
    let part_name = |format_id: &str| match request.choice {
        Choice::Video(v) if v.audio.as_deref() == Some(format_id) => "Sound",
        Choice::Video(_) => "Video",
        Choice::AudioMp3 { .. } => "Sound",
    };
    ytdlp.download(request, |event| match event {
        Event::Download(d) => {
            downloaded = true;
            let mut text = format!("{:<5} ", part_name(&d.format_id));
            text.push_str(&console::bar(d.fraction()));
            if let Some(fraction) = d.fraction() {
                text.push_str(&format!(" {:>3.0}%", fraction * 100.0));
            }
            if d.status == Status::Running {
                if let Some(speed) = d.speed {
                    text.push_str(&format!("  {}", units::speed(speed)));
                }
                if let Some(eta) = d.eta {
                    text.push_str(&format!("  {} left", units::duration(eta)));
                }
            }
            line.show(&text);
            if d.status == Status::Finished {
                line.finish();
            }
        }
        Event::Postprocess { status, step } => {
            if let (Status::Running, Some(words)) = (status, progress::describe_step(&step)) {
                line.show(&format!("{words}..."));
                line.finish();
            }
        }
        Event::Saved(path) => saved = Some(path),
    })?;
    line.finish();
    if !downloaded {
        // yt-dlp skips a file that is already in the folder.
        println!("The file was already in the folder.");
    }
    saved.ok_or_else(|| "yt-dlp finished without saying where the file is.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(words: &[&str]) -> Result<Option<Args>, String> {
        parse_args(words.iter().map(|w| w.to_string()))
    }

    #[test]
    fn takes_a_link_or_the_clipboard() {
        let args = parse(&["https://example.com/v/1"]).unwrap().unwrap();
        assert_eq!(args.link.as_deref(), Some("https://example.com/v/1"));
        assert!(parse(&["--clipboard"]).unwrap().unwrap().clipboard);
        assert!(parse(&["--clipboard", "https://example.com/v/1"]).is_err());
        assert!(parse(&["https://a.example", "https://b.example"]).is_err());
    }

    #[test]
    fn reads_the_settings_folder() {
        let args = parse(&["--dir", "C:\\test", "--choose-folder"])
            .unwrap()
            .unwrap();
        assert_eq!(args.dir, Some(PathBuf::from("C:\\test")));
        assert!(args.choose_folder);
        assert!(parse(&["--dir"]).is_err());
        assert!(parse(&["--frobnicate"]).is_err());
    }
}
