//! The menu built from what `yt-dlp -J` reports. `separate-streams.json` is
//! real output with names and links replaced; the other fixtures are small
//! cases written by hand.

use keepclip_core::info::{Size, VideoInfo};
use keepclip_core::menu::{self, Choice};

fn info(json: &str) -> VideoInfo {
    VideoInfo::parse(json).expect("fixture parses")
}

/// Each line as (label, yt-dlp `-f` argument, size).
fn lines(json: &str) -> Vec<(String, String, String)> {
    menu::build(&info(json))
        .iter()
        .map(|choice| {
            let selector = match choice {
                Choice::Video(v) => v.format_selector(),
                Choice::AudioMp3 { .. } => "mp3".to_string(),
            };
            (choice.label(), selector, choice.size_text())
        })
        .collect()
}

fn line(label: &str, selector: &str, size: &str) -> (String, String, String) {
    (label.to_string(), selector.to_string(), size.to_string())
}

#[test]
fn separate_video_and_sound_prefer_h264_with_aac() {
    assert_eq!(
        lines(include_str!("fixtures/separate-streams.json")),
        [
            // No H.264 above 1080p: the best video there is.
            line("2160p 60 fps", "401+140", "689.2 MB"),
            line("1440p 60 fps", "400+140", "307.9 MB"),
            // H.264 with a known size wins over the H.264 stream without one
            // and over VP9 and AV1, and the AAC sound over Opus and over the
            // DRC copies.
            line("1080p 60 fps", "299+140", "255.5 MB"),
            line("720p 60 fps", "298+140", "153.3 MB"),
            line("480p", "135+140", "36.7 MB"),
            line("360p", "134+140", "27.2 MB"),
            line("240p", "133+140", "18.7 MB"),
            line("144p", "160+140", "13.9 MB"),
            // 10:35 at 192 kbps.
            line("Audio only (MP3)", "mp3", "~14.5 MB"),
        ]
    );
}

#[test]
fn files_with_sound_skip_silent_video_and_use_the_shorter_side() {
    assert_eq!(
        lines(include_str!("fixtures/single-files.json")),
        [
            // Vertical 1080 × 1920 is 1080p. The silent 1080p video would
            // need separate sound, which this site does not have.
            line("1080p", "high", "~3.0 MB"),
            line("576p", "plain", "size unknown"),
            line("540p", "low", "1.0 MB"),
            line("Audio only (MP3)", "mp3", "~361 KB"),
        ]
    );
}

#[test]
fn unknown_sizes_say_so() {
    assert_eq!(
        lines(include_str!("fixtures/streams-without-sizes.json")),
        [
            line("1080p", "hls-1080+hls-audio-64000", "size unknown"),
            // The video has an estimate but the sound has none: the sum is
            // unknown.
            line("720p", "hls-720+hls-audio-64000", "size unknown"),
            line("480p", "hls-480+hls-audio-64000", "size unknown"),
            // No length, no estimate.
            line("Audio only (MP3)", "mp3", "size unknown"),
        ]
    );
}

#[test]
fn sound_only_sites_offer_only_mp3() {
    let menu = menu::build(&info(include_str!("fixtures/audio-only.json")));
    assert_eq!(
        menu,
        [Choice::AudioMp3 {
            size: Some(Size {
                bytes: 200 * 24_000,
                approximate: true
            })
        }]
    );
}

#[test]
fn a_format_without_codecs_counts_as_video_with_sound() {
    assert_eq!(
        lines(include_str!("fixtures/unknown-codecs.json")),
        [
            line("720p", "0", "size unknown"),
            line("Audio only (MP3)", "mp3", "size unknown"),
        ]
    );
}

#[test]
fn nothing_to_download_gives_an_empty_menu() {
    let empty = info(r#"{"id": "x", "formats": []}"#);
    assert!(menu::build(&empty).is_empty());
    let storyboards = info(
        r#"{"id": "x", "formats": [{"format_id": "sb0", "ext": "mhtml",
            "vcodec": "none", "acodec": "none", "format_note": "storyboard"}]}"#,
    );
    assert!(menu::build(&storyboards).is_empty());
}

#[test]
fn video_without_a_resolution_is_the_best_available() {
    let json = r#"{"id": "x", "formats": [
        {"format_id": "a", "vcodec": "vp9", "acodec": "none"},
        {"format_id": "b", "vcodec": "none", "acodec": "opus", "filesize": 10}
    ]}"#;
    assert_eq!(
        lines(json),
        [
            line("Best available", "a+b", "size unknown"),
            line("Audio only (MP3)", "mp3", "size unknown"),
        ]
    );
}

#[test]
fn frame_rates_below_50_are_not_shown() {
    let json = r#"{"id": "x", "formats": [
        {"format_id": "a", "vcodec": "avc1", "acodec": "mp4a", "height": 720, "fps": 29.97},
        {"format_id": "b", "vcodec": "avc1", "acodec": "mp4a", "height": 480, "fps": 59.94}
    ]}"#;
    let labels: Vec<String> = lines(json).into_iter().map(|(label, ..)| label).collect();
    assert_eq!(labels, ["720p", "480p 60 fps", "Audio only (MP3)"]);
}
