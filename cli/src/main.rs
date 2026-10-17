//! keepclip-cli: the console version of keepclip. It asks yt-dlp what the
//! link offers, shows a menu with the size of each choice, downloads the one
//! picked into the download folder and says where the file is.

mod console;
mod first_run;

use keepclip_core::download::Request;
use keepclip_core::info::VideoInfo;
use keepclip_core::menu::{self, Choice};
use keepclip_core::progress::{self, Event, Status};
use keepclip_core::{link, units};
use keepclip_system::args::{self, Args, Command};
use keepclip_system::files::{self, Moved};
use keepclip_system::stop::Stop;
use keepclip_system::ytdlp::YtDlp;
use keepclip_system::{Result, clipboard, notify, settings};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

/// The name the help gives the program.
const PROGRAM: &str = "keepclip-cli";

/// What the help says it does without a link.
const WITHOUT_LINK: [&str; 2] = ["without it, the link in the", "clipboard, or one you paste"];

fn main() -> ExitCode {
    let args = match args::parse(std::env::args().skip(1)) {
        Ok(Command::Run(args)) => args,
        Ok(Command::Help) => {
            print!("{}", args::help(PROGRAM, WITHOUT_LINK));
            return ExitCode::SUCCESS;
        }
        Ok(Command::Version) => {
            println!("keepclip {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("{PROGRAM}: {e}\n\n{}", args::help(PROGRAM, WITHOUT_LINK));
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
        first_run::choose_folder(&settings_file, &mut config)?;
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
        None => first_run::choose_folder(&settings_file, &mut config)?,
    };

    let ytdlp = first_run::ready_tools()?;
    println!("Reading the link...");
    let (info, json) = ytdlp.info(&url, &Stop::new())?;
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

    let work_folder = files::work_folder();
    for dir in [&folder, &work_folder] {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Could not create the folder {}: {e}", dir.display()))?;
    }
    let request = Request {
        url: &url,
        choice,
        work_folder: &work_folder,
        playlist_item,
    };

    // Ask before downloading, so a "no" costs no time. If yt-dlp cannot say
    // the name, the question comes when the file is ready instead.
    let mut may_replace = false;
    if let Some(name) = ytdlp.final_name(&json, &request)
        && folder.join(&name).exists()
    {
        if !ask_to_replace(&name)? {
            println!("Kept the file that was there.");
            return Ok(Finish::Nothing);
        }
        may_replace = true;
    }

    println!("\nSaving to {}", folder.display());
    let finished = PathBuf::from(download(&ytdlp, &request)?);
    let Some(saved) = move_into(&finished, &folder, may_replace)? else {
        return Ok(Finish::Nothing);
    };
    println!("\nSaved: {}", saved.display());
    let notified = notify::saved(&saved);
    Ok(Finish::Saved { notified })
}

fn ask_to_replace(name: &str) -> Result<bool> {
    println!("\nThe folder already has \"{name}\".");
    let answer = console::ask("Replace it? It stays as it is until the new one is ready. (y/N) ")?;
    Ok(answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes"))
}

/// Moves the finished file into `folder`. A file with the same name is
/// replaced only when `may_replace`, or when the user says so now;
/// otherwise the new file is deleted and `None` returned.
fn move_into(file: &Path, folder: &Path, may_replace: bool) -> Result<Option<PathBuf>> {
    let target = match files::move_into(file, folder, may_replace)? {
        Moved::Saved(target) => return Ok(Some(target)),
        Moved::Exists(target) => target,
    };
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    if !ask_to_replace(&name)? {
        let _ = std::fs::remove_file(file);
        println!("Kept the file that was there; the new download was deleted.");
        return Ok(None);
    }
    match files::move_into(file, folder, true)? {
        Moved::Saved(target) => Ok(Some(target)),
        Moved::Exists(_) => Err("Could not replace the file.".into()),
    }
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
    if let Some(author) = video.author() {
        line.push_str(&format!(" | {author}"));
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
fn download(ytdlp: &YtDlp, request: &Request<'_>) -> Result<String> {
    let mut line = console::StatusLine::default();
    let mut saved = None;
    let mut downloaded = false;
    ytdlp.download(request, &Stop::new(), |event| match event {
        Event::Download(d) => {
            downloaded = true;
            let mut text = format!("{:<5} ", request.choice.part_name(&d.format_id));
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
        // yt-dlp skips a file it finished before, left in the work folder
        // when keepclip was closed before moving it.
        println!("The file was ready from an earlier try.");
    }
    saved.ok_or_else(|| "yt-dlp finished without saying where the file is.".into())
}
