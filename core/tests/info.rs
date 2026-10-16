//! Reading what `yt-dlp -J` prints.

use keepclip_core::info::VideoInfo;

#[test]
fn reads_a_video() {
    let video = VideoInfo::parse(include_str!("fixtures/separate-streams.json")).unwrap();
    assert_eq!(video.id, "sample1");
    assert_eq!(video.title.as_deref(), Some("Sample video"));
    assert_eq!(video.uploader.as_deref(), Some("Sample channel"));
    assert_eq!(video.duration, Some(635.0));
    assert!(!video.is_playlist());
    assert_eq!(video.formats.len(), 53);
}

#[test]
fn accepts_numbers_where_text_is_usual_and_decimals_in_sizes() {
    let stream = VideoInfo::parse(include_str!("fixtures/streams-without-sizes.json")).unwrap();
    assert_eq!(stream.id, "1234567890123");
    let clip = VideoInfo::parse(include_str!("fixtures/single-files.json")).unwrap();
    assert_eq!(clip.formats[2].filesize_approx, Some(3_145_728));
    assert_eq!(
        clip.title.as_deref(),
        Some("Sample clip #tag #another 🎬✨")
    );
}

#[test]
fn reads_the_videos_of_a_playlist_and_skips_hidden_ones() {
    let post = VideoInfo::parse(include_str!("fixtures/playlist.json")).unwrap();
    assert!(post.is_playlist());
    let videos: Vec<(&str, Option<u64>)> = post
        .videos()
        .map(|v| (v.id.as_str(), v.playlist_index))
        .collect();
    assert_eq!(videos, [("sample31", Some(1)), ("sample33", Some(3))]);
}

#[test]
fn unknown_fields_and_nulls_are_fine() {
    let json = r#"{"id": "x", "title": null, "duration": null, "extra": {"a": [1, 2]},
        "formats": [{"format_id": "f", "height": null, "fps": "NA", "url": "https://e.example"}]}"#;
    let video = VideoInfo::parse(json).unwrap();
    assert_eq!(video.title, None);
    assert_eq!(video.formats[0].height, None);
    assert_eq!(video.formats[0].fps, None);
}

#[test]
fn something_else_is_an_error() {
    assert!(VideoInfo::parse("").is_err());
    assert!(VideoInfo::parse("[]").is_err());
    assert!(VideoInfo::parse(r#"{"title": "no id"}"#).is_err());
}
