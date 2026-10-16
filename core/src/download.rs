//! The command lines keepclip gives yt-dlp: one to read what a link offers,
//! one to learn the name the file will have, one to download it.
//!
//! yt-dlp makes every file in a work folder of keepclip's, and keepclip moves
//! the finished file to the download folder. So yt-dlp never sees what is in
//! the download folder: an MP3 made from a video starts as a file with that
//! video's name, and yt-dlp, finding the video there, would take it as
//! already downloaded and delete it after the conversion. keepclip checks the
//! name first and asks before replacing a file.

use crate::menu::{Choice, MP3_KBPS};
use crate::progress;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Name of the saved file: who posted it, the title and the video id, which
/// keeps two videos with the same title apart. Some sites leave the uploader
/// empty and give the channel; yt-dlp skips empty fields in a list.
pub const FILE_NAME: &str = "%(uploader,channel|Unknown)s - %(title)s [%(id)s].%(ext)s";

/// Longest file name, without the extension, in characters. Titles can be a
/// whole post, and Windows refuses paths longer than 260 characters.
pub const MAX_NAME_CHARS: u32 = 120;

/// How many videos of a playlist keepclip reads. Reading each one takes a
/// moment, and the menu is not made for long lists.
pub const PLAYLIST_LIMIT: u32 = 20;

/// Where yt-dlp finds the programs it needs. `None` leaves the search to
/// yt-dlp, which looks in the PATH.
#[derive(Debug, Clone, Default)]
pub struct Tools {
    /// ffmpeg, or the folder that holds it.
    pub ffmpeg: Option<PathBuf>,
    /// The Deno executable, which runs the JavaScript some sites need.
    pub deno: Option<PathBuf>,
}

/// What to download.
#[derive(Debug, Clone)]
pub struct Request<'a> {
    pub url: &'a str,
    pub choice: &'a Choice,
    /// Where yt-dlp makes the file, parts and all, before keepclip moves it
    /// to the download folder.
    pub work_folder: &'a Path,
    /// Position of the chosen video in a playlist, counting from 1.
    pub playlist_item: Option<u64>,
}

/// Arguments for `yt-dlp -J`, which prints what the link offers as JSON.
pub fn info_args(url: &str, tools: &Tools) -> Vec<OsString> {
    let mut args = common(tools);
    args.extend(
        [
            "--playlist-items".to_string(),
            format!("1:{PLAYLIST_LIMIT}"),
            "-J".to_string(),
            "--".to_string(),
            url.to_string(),
        ]
        .map(OsString::from),
    );
    args
}

/// Arguments for printing the path the download would have, without
/// downloading, from the JSON `yt-dlp -J` printed, saved in `info_json`. The
/// path is the one before any conversion: see [`final_file_name`].
pub fn name_args(request: &Request<'_>, info_json: &Path, tools: &Tools) -> Vec<OsString> {
    let mut args = common(tools);
    args.extend(of_the_file(request));
    args.extend(["--simulate", "--print", "filename", "--load-info-json"].map(OsString::from));
    args.push(info_json.as_os_str().to_owned());
    args
}

/// Arguments for downloading a choice of the menu, with progress lines that
/// [`progress::parse`] reads.
pub fn download_args(request: &Request<'_>, tools: &Tools) -> Vec<OsString> {
    let mut args = common(tools);
    let mut push = |arg: &str| args.push(OsString::from(arg));
    push("--newline");
    push("--progress");
    push("--progress-template");
    push(&progress::download_template());
    push("--progress-template");
    push(&progress::postprocess_template());
    push("--print");
    push(&progress::saved_template());
    args.extend(of_the_file(request));
    args.push(OsString::from("--"));
    args.push(OsString::from(request.url));
    args
}

/// The name of the file a download printed by [`name_args`] ends up with:
/// the last line, without its folder, and with `.mp3` for an MP3, which
/// yt-dlp makes from that file after the download.
pub fn final_file_name(printed: &str, choice: &Choice) -> Option<String> {
    let path = printed
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())?;
    let name = path.rsplit(['\\', '/']).next().filter(|n| !n.is_empty())?;
    Some(match choice {
        Choice::Video(_) => name.to_string(),
        Choice::AudioMp3 { .. } => {
            let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
            format!("{stem}.mp3")
        }
    })
}

/// What decides the file: the formats, the conversion, the name and the
/// folder. The same for the name check and for the download.
fn of_the_file(request: &Request<'_>) -> Vec<OsString> {
    let mut args: Vec<String> = vec![
        "-o".into(),
        FILE_NAME.into(),
        "--trim-filenames".into(),
        MAX_NAME_CHARS.to_string(),
        "--windows-filenames".into(),
    ];
    match request.choice {
        Choice::Video(video) => args.extend([
            "-f".into(),
            video.format_selector(),
            "--merge-output-format".into(),
            video.merge_into.into(),
        ]),
        Choice::AudioMp3 { source, .. } => args.extend([
            "-f".into(),
            source.clone(),
            "-x".into(),
            "--audio-format".into(),
            "mp3".into(),
            "--audio-quality".into(),
            format!("{MP3_KBPS}K"),
        ]),
    }
    if let Some(item) = request.playlist_item {
        args.extend(["--playlist-items".into(), item.to_string()]);
    }
    let mut args: Vec<OsString> = args.into_iter().map(OsString::from).collect();
    let mut folder = OsString::from("home:");
    folder.push(request.work_folder);
    args.extend([OsString::from("-P"), folder]);
    args
}

fn common(tools: &Tools) -> Vec<OsString> {
    let mut args: Vec<OsString> = [
        // A yt-dlp configuration file of the user could change the output
        // keepclip reads.
        "--ignore-config",
        // A link to a video inside a playlist means that video.
        "--no-playlist",
        "--color",
        "never",
        // File names may have any character; the default encoding of a pipe
        // on Windows would drop some.
        "--encoding",
        "utf-8",
    ]
    .map(OsString::from)
    .into();
    if let Some(ffmpeg) = &tools.ffmpeg {
        args.push(OsString::from("--ffmpeg-location"));
        args.push(ffmpeg.as_os_str().to_owned());
    }
    if let Some(deno) = &tools.deno {
        let mut runtime = OsString::from("deno:");
        runtime.push(deno.as_os_str());
        args.push(OsString::from("--js-runtimes"));
        args.push(runtime);
    }
    args
}
