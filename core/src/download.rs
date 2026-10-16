//! The command lines keepclip gives yt-dlp: one to read what a link offers,
//! one to download the choice.

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
    pub folder: &'a Path,
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
    push("-o");
    push(FILE_NAME);
    push("--trim-filenames");
    push(&MAX_NAME_CHARS.to_string());
    push("--windows-filenames");
    match request.choice {
        Choice::Video(video) => {
            push("-f");
            push(&video.format_selector());
            push("--merge-output-format");
            push(video.merge_into);
        }
        Choice::AudioMp3 { source, .. } => {
            push("-f");
            push(source);
            push("-x");
            push("--audio-format");
            push("mp3");
            push("--audio-quality");
            push(&format!("{MP3_KBPS}K"));
        }
    }
    if let Some(item) = request.playlist_item {
        push("--playlist-items");
        push(&item.to_string());
    }
    args.push(OsString::from("-P"));
    args.push(request.folder.as_os_str().to_owned());
    args.push(OsString::from("--"));
    args.push(OsString::from(request.url));
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
