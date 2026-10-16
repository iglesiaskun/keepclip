//! The command lines given to yt-dlp.

use keepclip_core::download::{self, Request, Tools};
use keepclip_core::menu::{Choice, VideoChoice};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

fn text(args: Vec<OsString>) -> Vec<String> {
    args.into_iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

/// The value right after `option`, if it is there.
fn value<'a>(args: &'a [String], option: &str) -> Option<&'a str> {
    let at = args.iter().position(|a| a == option)?;
    args.get(at + 1).map(String::as_str)
}

fn tools() -> Tools {
    Tools {
        ffmpeg: Some(PathBuf::from("C:\\keepclip\\bin\\ffmpeg")),
        deno: Some(PathBuf::from("C:\\keepclip\\bin\\deno.exe")),
    }
}

fn video_1080() -> Choice {
    Choice::Video(VideoChoice {
        resolution: Some(1080),
        fps: Some(60.0),
        video: "299".to_string(),
        audio: Some("140".to_string()),
        size: None,
    })
}

#[test]
fn reading_a_link_prints_json_and_ends_with_the_link() {
    let args = text(download::info_args("https://v.example/1", &tools()));
    assert!(args.contains(&"-J".to_string()));
    assert!(args.contains(&"--ignore-config".to_string()));
    assert!(args.contains(&"--no-playlist".to_string()));
    assert_eq!(value(&args, "--playlist-items"), Some("1:20"));
    // `--` keeps a link from ever being read as an option.
    assert_eq!(&args[args.len() - 2..], ["--", "https://v.example/1"]);
}

#[test]
fn the_tools_are_passed_when_known() {
    let args = text(download::info_args("https://v.example/1", &tools()));
    assert_eq!(
        value(&args, "--ffmpeg-location"),
        Some("C:\\keepclip\\bin\\ffmpeg")
    );
    assert_eq!(
        value(&args, "--js-runtimes"),
        Some("deno:C:\\keepclip\\bin\\deno.exe")
    );
    let args = text(download::info_args(
        "https://v.example/1",
        &Tools::default(),
    ));
    assert_eq!(value(&args, "--ffmpeg-location"), None);
    assert_eq!(value(&args, "--js-runtimes"), None);
}

#[test]
fn a_video_joins_the_two_formats_into_mp4_or_mkv() {
    let choice = video_1080();
    let request = Request {
        url: "https://v.example/1",
        choice: &choice,
        folder: Path::new("D:\\Videos"),
        playlist_item: None,
    };
    let args = text(download::download_args(&request, &tools()));
    assert_eq!(value(&args, "-f"), Some("299+140"));
    assert_eq!(value(&args, "--merge-output-format"), Some("mp4/mkv"));
    assert_eq!(value(&args, "-P"), Some("D:\\Videos"));
    assert!(!args.contains(&"-x".to_string()));
    assert_eq!(value(&args, "--playlist-items"), None);
    assert_eq!(&args[args.len() - 2..], ["--", "https://v.example/1"]);
}

#[test]
fn files_are_named_after_the_uploader_title_and_id() {
    let choice = video_1080();
    let request = Request {
        url: "https://v.example/1",
        choice: &choice,
        folder: Path::new("D:\\Videos"),
        playlist_item: None,
    };
    let args = text(download::download_args(&request, &tools()));
    assert_eq!(
        value(&args, "-o"),
        Some("%(uploader)s - %(title)s [%(id)s].%(ext)s")
    );
    assert_eq!(value(&args, "--trim-filenames"), Some("120"));
    assert!(args.contains(&"--windows-filenames".to_string()));
}

#[test]
fn progress_comes_one_line_at_a_time_in_keepclip_templates() {
    let choice = video_1080();
    let request = Request {
        url: "https://v.example/1",
        choice: &choice,
        folder: Path::new("D:\\Videos"),
        playlist_item: None,
    };
    let args = text(download::download_args(&request, &tools()));
    assert!(args.contains(&"--newline".to_string()));
    let templates: Vec<&String> = args
        .iter()
        .zip(args.iter().skip(1))
        .filter(|(option, _)| *option == "--progress-template")
        .map(|(_, template)| template)
        .collect();
    assert_eq!(templates.len(), 2);
    assert!(templates[0].starts_with("download:keepclip-progress "));
    assert!(templates[1].starts_with("postprocess:keepclip-postprocess "));
    assert_eq!(
        value(&args, "--print"),
        Some("after_move:keepclip-file %(filepath)s")
    );
}

#[test]
fn audio_only_converts_the_best_sound_to_mp3() {
    let choice = Choice::AudioMp3 { size: None };
    let request = Request {
        url: "https://v.example/1",
        choice: &choice,
        folder: Path::new("D:\\Music"),
        playlist_item: None,
    };
    let args = text(download::download_args(&request, &tools()));
    assert_eq!(value(&args, "-f"), Some("ba/b"));
    assert!(args.contains(&"-x".to_string()));
    assert_eq!(value(&args, "--audio-format"), Some("mp3"));
    assert_eq!(value(&args, "--audio-quality"), Some("192K"));
    assert_eq!(value(&args, "--merge-output-format"), None);
}

#[test]
fn a_video_of_a_playlist_names_its_position() {
    let choice = video_1080();
    let request = Request {
        url: "https://v.example/post/1",
        choice: &choice,
        folder: Path::new("D:\\Videos"),
        playlist_item: Some(3),
    };
    let args = text(download::download_args(&request, &tools()));
    assert_eq!(value(&args, "--playlist-items"), Some("3"));
}
