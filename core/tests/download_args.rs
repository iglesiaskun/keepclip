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

/// Every value given to `option`, in order.
fn values(args: &[String], option: &str) -> Vec<String> {
    args.iter()
        .zip(args.iter().skip(1))
        .filter(|(o, _)| *o == option)
        .map(|(_, v)| v.clone())
        .collect()
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
        merge_into: "mp4/mkv",
    })
}

fn mp3(source: &str) -> Choice {
    Choice::AudioMp3 {
        size: None,
        source: source.to_string(),
    }
}

fn request(choice: &Choice) -> Request<'_> {
    Request {
        url: "https://v.example/1",
        choice,
        work_folder: Path::new("C:\\Temp\\keepclip"),
        playlist_item: None,
    }
}

fn download(choice: &Choice) -> Vec<String> {
    text(download::download_args(&request(choice), &tools()))
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
fn a_video_joins_its_two_formats() {
    let args = download(&video_1080());
    assert_eq!(value(&args, "-f"), Some("299+140"));
    assert_eq!(value(&args, "--merge-output-format"), Some("mp4/mkv"));
    assert!(!args.contains(&"-x".to_string()));
    assert_eq!(value(&args, "--playlist-items"), None);
    assert_eq!(&args[args.len() - 2..], ["--", "https://v.example/1"]);
}

#[test]
fn every_file_is_made_in_the_work_folder() {
    // yt-dlp never sees the download folder: an MP3 made there from a video
    // already saved would take that video as its download and delete it.
    for choice in [video_1080(), mp3("video-with-sound")] {
        assert_eq!(
            values(&download(&choice), "-P"),
            ["home:C:\\Temp\\keepclip"]
        );
    }
}

#[test]
fn files_are_named_after_the_uploader_title_and_id() {
    let args = download(&video_1080());
    // The channel stands in for an empty uploader.
    assert_eq!(
        value(&args, "-o"),
        Some("%(uploader,channel|Unknown)s - %(title)s [%(id)s].%(ext)s")
    );
    assert_eq!(value(&args, "--trim-filenames"), Some("120"));
    assert!(args.contains(&"--windows-filenames".to_string()));
}

#[test]
fn progress_comes_one_line_at_a_time_in_keepclip_templates() {
    let args = download(&video_1080());
    assert!(args.contains(&"--newline".to_string()));
    let templates = values(&args, "--progress-template");
    assert_eq!(templates.len(), 2);
    assert!(templates[0].starts_with("download:keepclip-progress "));
    assert!(templates[1].starts_with("postprocess:keepclip-postprocess "));
    assert_eq!(
        value(&args, "--print"),
        Some("after_move:keepclip-file %(filepath)s")
    );
}

#[test]
fn audio_only_converts_its_source_to_mp3() {
    for source in ["ba/b", "video-with-sound"] {
        let args = download(&mp3(source));
        assert_eq!(value(&args, "-f"), Some(source));
        assert!(args.contains(&"-x".to_string()));
        assert_eq!(value(&args, "--audio-format"), Some("mp3"));
        assert_eq!(value(&args, "--audio-quality"), Some("192K"));
        assert_eq!(value(&args, "--merge-output-format"), None);
    }
}

#[test]
fn a_video_of_a_playlist_names_its_position() {
    let choice = video_1080();
    let request = Request {
        playlist_item: Some(3),
        ..request(&choice)
    };
    let args = text(download::download_args(&request, &tools()));
    assert_eq!(value(&args, "--playlist-items"), Some("3"));
}

#[test]
fn the_name_check_reads_the_saved_json_and_downloads_nothing() {
    let choice = video_1080();
    let request = Request {
        playlist_item: Some(3),
        ..request(&choice)
    };
    let json = Path::new("C:\\Temp\\keepclip\\1234.info.json");
    let args = text(download::name_args(&request, json, &tools()));
    assert!(args.contains(&"--simulate".to_string()));
    assert_eq!(value(&args, "--print"), Some("filename"));
    assert_eq!(
        value(&args, "--load-info-json"),
        Some("C:\\Temp\\keepclip\\1234.info.json")
    );
    assert!(!args.contains(&"--".to_string()));
    assert!(!args.contains(&"https://v.example/1".to_string()));
    // Whatever decides the name is the same as in the download.
    let download = text(download::download_args(&request, &tools()));
    for option in [
        "-o",
        "--trim-filenames",
        "-f",
        "--merge-output-format",
        "--playlist-items",
        "-P",
    ] {
        assert_eq!(values(&args, option), values(&download, option), "{option}");
    }
}

#[test]
fn the_final_name_is_the_printed_one_without_its_folder() {
    let printed = "C:\\Temp\\keepclip\\Someone - A title [abc].mp4\n";
    assert_eq!(
        download::final_file_name(printed, &video_1080()).as_deref(),
        Some("Someone - A title [abc].mp4")
    );
    assert_eq!(
        download::final_file_name("/tmp/keepclip/A [x].mkv", &video_1080()).as_deref(),
        Some("A [x].mkv")
    );
}

#[test]
fn an_mp3_takes_the_name_of_its_source_with_mp3() {
    let printed = "C:\\Temp\\keepclip\\Mr. Someone - v1.2 [abc].m4a\r\n";
    assert_eq!(
        download::final_file_name(printed, &mp3("ba/b")).as_deref(),
        Some("Mr. Someone - v1.2 [abc].mp3")
    );
}

#[test]
fn nothing_printed_gives_no_name() {
    assert_eq!(download::final_file_name("", &video_1080()), None);
    assert_eq!(download::final_file_name("\n  \n", &video_1080()), None);
    assert_eq!(download::final_file_name("C:\\Temp\\", &video_1080()), None);
}
