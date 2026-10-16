//! Following a download through the lines yt-dlp prints with keepclip's
//! templates. The fixture holds lines yt-dlp printed for a video with
//! separate sound: video, sound, joining, moving, saved.

use keepclip_core::info::Size;
use keepclip_core::progress::{self, Event, Status, describe_step, parse};

const LINES: &str = include_str!("fixtures/progress-lines.txt");

fn events() -> Vec<Event> {
    LINES.lines().filter_map(parse).collect()
}

#[test]
fn every_line_of_a_real_download_is_understood() {
    assert_eq!(events().len(), LINES.lines().count());
}

#[test]
fn reads_the_download_fields() {
    let Event::Download(first) = &events()[2] else {
        panic!("not a download line");
    };
    assert_eq!(first.status, Status::Running);
    assert_eq!(first.format_id, "160");
    assert_eq!(first.downloaded, Some(7168));
    assert_eq!(
        first.total,
        Some(Size {
            bytes: 4_323_893,
            approximate: false
        })
    );
    assert_eq!(first.speed.map(|s| s.round()), Some(3_583_405.0));
    assert_eq!(first.eta, Some(1));
}

#[test]
fn unknown_fields_are_none() {
    let Event::Download(start) = &events()[0] else {
        panic!("not a download line");
    };
    assert_eq!(start.speed, None);
    assert_eq!(start.eta, None);
}

#[test]
fn an_estimated_total_is_marked_as_such() {
    let Event::Download(sound) = &events()[5] else {
        panic!("not a download line");
    };
    assert_eq!(sound.format_id, "139");
    assert_eq!(
        sound.total,
        Some(Size {
            bytes: 3_871_021,
            approximate: true
        })
    );
}

#[test]
fn the_fraction_follows_the_bytes() {
    let fractions: Vec<Option<f64>> = events()
        .iter()
        .filter_map(|e| match e {
            Event::Download(d) => Some(d.fraction()),
            _ => None,
        })
        .collect();
    assert_eq!(fractions[0].map(|f| (f * 1000.0).round()), Some(0.0));
    assert_eq!(fractions[3].map(|f| (f * 100.0).round()), Some(50.0));
    assert_eq!(fractions[4], Some(1.0));
}

#[test]
fn the_steps_after_the_download() {
    let steps: Vec<(Status, String)> = events()
        .into_iter()
        .filter_map(|e| match e {
            Event::Postprocess { status, step } => Some((status, step)),
            _ => None,
        })
        .collect();
    assert_eq!(steps[0], (Status::Running, "Merger".to_string()));
    assert_eq!(steps[1], (Status::Finished, "Merger".to_string()));
    assert_eq!(describe_step("Merger"), Some("Joining video and sound"));
    assert_eq!(describe_step("ExtractAudio"), Some("Converting to MP3"));
    assert_eq!(describe_step("FixupM3u8"), Some("Fixing the file"));
    assert_eq!(describe_step("MoveFiles"), None);
}

#[test]
fn the_saved_path_may_have_spaces_and_brackets() {
    assert_eq!(
        events().last(),
        Some(&Event::Saved(
            "C:\\Downloads\\Sample channel - Sample video [sample1].mp4".to_string()
        ))
    );
}

#[test]
fn other_lines_are_not_events() {
    assert_eq!(parse("WARNING: something"), None);
    assert_eq!(parse("ERROR: Unsupported URL: https://example.com"), None);
    assert_eq!(parse("keepclip-progress"), None);
    assert_eq!(parse("keepclip-progress downloading 1 2"), None);
    assert_eq!(parse("keepclip-file "), None);
    assert_eq!(parse(""), None);
}

#[test]
fn a_format_id_with_a_space_stays_whole() {
    let Some(Event::Download(d)) = parse("keepclip-progress downloading 1 2 NA NA NA hls 720p")
    else {
        panic!("not a download line");
    };
    assert_eq!(d.format_id, "hls 720p");
}

#[test]
fn windows_line_endings_are_ignored() {
    assert_eq!(
        parse("keepclip-file C:\\a.mp4\r\n"),
        Some(Event::Saved("C:\\a.mp4".to_string()))
    );
}

#[test]
fn templates_start_with_the_words_parse_looks_for() {
    for (template, kind) in [
        (progress::download_template(), "download:keepclip-progress "),
        (
            progress::postprocess_template(),
            "postprocess:keepclip-postprocess ",
        ),
        (progress::saved_template(), "after_move:keepclip-file "),
    ] {
        assert!(template.starts_with(kind), "{template}");
    }
}
