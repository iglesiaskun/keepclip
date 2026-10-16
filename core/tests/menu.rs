//! The menu built from what `yt-dlp -J` reports. Five fixtures are real
//! output from different sites, with names, ids and links replaced:
//! `separate-streams`, `vertical-extra-sizes`, `streams-and-one-file`,
//! `empty-uploader` and `h264-only-low`. The others are small cases written
//! by hand.

use keepclip_core::info::{Size, VideoInfo};
use keepclip_core::menu::{self, Choice};

fn info(json: &str) -> VideoInfo {
    VideoInfo::parse(json).expect("fixture parses")
}

/// Each line as (label, yt-dlp `-f` argument, size). For the MP3, the
/// argument is written "mp3 from <-f>".
fn lines(json: &str) -> Vec<(String, String, String)> {
    menu::build(&info(json))
        .iter()
        .map(|choice| {
            let selector = match choice {
                Choice::Video(v) => v.format_selector(),
                Choice::AudioMp3 { source, .. } => format!("mp3 from {source}"),
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
            line("Audio only (MP3)", "mp3 from ba/b", "~14.5 MB"),
        ]
    );
}

#[test]
fn resolutions_without_h264_below_the_highest_h264_are_left_out() {
    // This vertical video also has 608 × 1080 in VP9 and AV1, between 720p
    // and 480p. With H.264 up to 1080p, that line would only play in fewer
    // places.
    assert_eq!(
        lines(include_str!("fixtures/vertical-extra-sizes.json")),
        [
            line("1080p 60 fps", "299+140", "15.9 MB"),
            line("720p 60 fps", "298+140", "9.7 MB"),
            line("480p", "135+140", "2.4 MB"),
            line("360p", "134+140", "1.7 MB"),
            line("240p", "133+140", "854 KB"),
            line("144p", "160+140", "568 KB"),
            line("Audio only (MP3)", "mp3 from ba/b", "~539 KB"),
        ]
    );
}

#[test]
fn resolutions_above_the_highest_h264_stay_in_another_codec() {
    // H.264 only at 576 × 1024; 720p and 1080p exist in another codec only.
    assert_eq!(
        lines(include_str!("fixtures/h264-only-low.json")),
        [
            line("1080p", "hevc_1080p_951110-1", "1.3 MB"),
            line("720p", "hevc_720p_679534-1", "935 KB"),
            line("576p", "h264_540p_379463-1", "522 KB"),
            // The video carries its sound, so the MP3 comes from it, not from the
            // separate track this site lists, which is the whole song.
            line(
                "Audio only (MP3)",
                "mp3 from hevc_1080p_951110-1",
                "~258 KB",
            ),
        ]
    );
}

#[test]
fn without_sizes_the_bitrate_and_length_give_an_estimate() {
    // 9.5 s at 213 kbps of video and 32 kbps of sound. The single file with
    // sound has no known codec, so the H.264 stream wins.
    assert_eq!(
        lines(include_str!("fixtures/streams-and-one-file.json")),
        [
            line("368p", "hls-213+hls-audio-32000-Audio", "~285 KB"),
            line("Audio only (MP3)", "mp3 from ba/b", "~223 KB"),
        ]
    );
}

/// `--merge-output-format` of each video line.
fn merges(json: &str) -> Vec<&'static str> {
    menu::build(&info(json))
        .iter()
        .filter_map(|choice| match choice {
            Choice::Video(v) => Some(v.merge_into),
            Choice::AudioMp3 { .. } => None,
        })
        .collect()
}

#[test]
fn parts_that_both_come_in_mp4_are_joined_into_mp4() {
    // Codecs yt-dlp does not know, or VP9, would make it pick MKV; parts
    // that are MP4 files already always fit in one.
    assert_eq!(
        merges(include_str!("fixtures/streams-and-one-file.json")),
        ["mp4"]
    );
    assert_eq!(
        merges(include_str!("fixtures/empty-uploader.json")),
        ["mp4"]
    );
    let webm = r#"{"id": "x", "formats": [
        {"format_id": "a", "ext": "webm", "vcodec": "vp9", "acodec": "none", "height": 1440},
        {"format_id": "b", "ext": "m4a", "vcodec": "none", "acodec": "mp4a.40.2"}
    ]}"#;
    assert_eq!(merges(webm), ["mp4/mkv"]);
}

#[test]
fn without_a_length_a_bitrate_gives_no_size() {
    assert_eq!(
        lines(include_str!("fixtures/empty-uploader.json")),
        [
            line(
                "720p",
                "dash-0000000000000004v+dash-0000000000000001a",
                "size unknown"
            ),
            line("Audio only (MP3)", "mp3 from ba/b", "size unknown"),
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
            line("Audio only (MP3)", "mp3 from high", "~361 KB"),
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
            line("Audio only (MP3)", "mp3 from ba/b", "size unknown"),
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
            }),
            source: "ba/b".to_string(),
        }]
    );
}

#[test]
fn a_format_without_codecs_counts_as_video_with_sound() {
    assert_eq!(
        lines(include_str!("fixtures/unknown-codecs.json")),
        [
            line("720p", "0", "size unknown"),
            line("Audio only (MP3)", "mp3 from 0", "size unknown"),
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
            line("Audio only (MP3)", "mp3 from ba/b", "size unknown"),
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
