//! What the interface shows, made from what yt-dlp reported: plain data that
//! travels to the interface as JSON, with sizes and times already in words.

use keepclip_core::info::VideoInfo;
use keepclip_core::menu::{self, Choice};
use keepclip_core::progress::{self, Event, Status};
use keepclip_core::units;
use keepclip_system::stop::Stopped;
use keepclip_system::ytdlp::Failed;
use serde::Serialize;

/// A video of the link, with its menu and its place in the list.
pub struct Video {
    pub info: VideoInfo,
    /// Position in the list for yt-dlp, counting from 1.
    pub playlist_item: Option<u64>,
    pub choices: Vec<Choice>,
}

/// The videos to choose from, as the console's `pick_video` sees them:
/// the video, or the downloadable videos of a list. Videos with nothing to
/// download are left out.
pub fn videos(info: &VideoInfo) -> Result<Vec<Video>, &'static str> {
    let videos: Vec<Video> = if !info.is_playlist() {
        vec![Video {
            info: info.clone(),
            playlist_item: None,
            choices: menu::build(info),
        }]
    } else {
        let entries: Vec<&VideoInfo> = info.videos().collect();
        let only_one = entries.len() == 1;
        entries
            .into_iter()
            .enumerate()
            .map(|(n, video)| Video {
                info: video.clone(),
                // Without its position, yt-dlp would download the whole list.
                playlist_item: if only_one {
                    video.playlist_index
                } else {
                    Some(video.playlist_index.unwrap_or(n as u64 + 1))
                },
                choices: menu::build(video),
            })
            .collect()
    };
    let videos: Vec<Video> = videos
        .into_iter()
        .filter(|v| !v.choices.is_empty())
        .collect();
    if videos.is_empty() {
        return Err("yt-dlp found nothing to download at this link.");
    }
    Ok(videos)
}

/// A link that was read: one video, or several to pick from.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkView {
    /// Which reading this is, to match the thumbnails that come later.
    pub read: u64,
    pub author: Option<String>,
    pub videos: Vec<VideoView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoView {
    pub title: String,
    pub author: Option<String>,
    /// "10:35".
    pub duration: Option<String>,
    /// "2 of 3", for a video of a list.
    pub position: Option<String>,
    pub choices: Vec<ChoiceView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceView {
    /// "1080p 60 fps", "Audio only (MP3)".
    pub label: String,
    /// "48.2 MB", "~48.2 MB" or "size unknown".
    pub size: String,
    pub estimated: bool,
    pub unknown: bool,
    /// The size in bytes, for the progress of the whole download.
    pub bytes: Option<u64>,
    pub mp3: bool,
    /// "Video", "Sound": what yt-dlp downloads, in order.
    pub parts: &'static [&'static str],
    /// "Joining video and sound", "Converting to MP3".
    pub last_step: Option<&'static str>,
}

pub fn link(read: u64, list: &VideoInfo, videos: &[Video]) -> LinkView {
    let count = videos.len();
    LinkView {
        read,
        author: list
            .author()
            .or_else(|| videos.first().and_then(|v| v.info.author()))
            .map(str::to_string),
        videos: videos
            .iter()
            .enumerate()
            .map(|(n, video)| VideoView {
                title: video
                    .info
                    .title
                    .clone()
                    .unwrap_or_else(|| video.info.id.clone()),
                author: video.info.author().map(str::to_string),
                duration: video.info.duration.map(|s| units::duration(s as u64)),
                position: (count > 1).then(|| format!("{} of {count}", n + 1)),
                choices: video.choices.iter().map(choice).collect(),
            })
            .collect(),
    }
}

fn choice(choice: &Choice) -> ChoiceView {
    let size = choice.size();
    ChoiceView {
        label: choice.label(),
        size: choice.size_text(),
        estimated: size.is_some_and(|s| s.approximate),
        unknown: size.is_none(),
        bytes: size.map(|s| s.bytes),
        mp3: matches!(choice, Choice::AudioMp3 { .. }),
        parts: choice.parts(),
        last_step: choice.last_step(),
    }
}

/// One update of the download.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Progress {
    /// A part ("Video", "Sound") downloading or finished.
    #[serde(rename_all = "camelCase")]
    Part {
        part: &'static str,
        finished: bool,
        /// From 0 to 1, when the total is known.
        fraction: Option<f64>,
        /// Bytes so far, for the progress of the whole download.
        bytes: Option<u64>,
        done: Option<String>,
        /// "48.2 MB", or "~48.2 MB" while yt-dlp estimates it.
        total: Option<String>,
        speed: Option<String>,
        /// Time left: "0:42".
        left: Option<String>,
    },
    /// A step after the download, running.
    Step { words: &'static str },
}

/// What a line of yt-dlp means for the interface, or `None` for lines it
/// does not show.
pub fn progress(choice: &Choice, event: &Event) -> Option<Progress> {
    match event {
        Event::Download(d) => {
            let finished = d.status == Status::Finished;
            let running = d.status == Status::Running;
            let bytes = d
                .downloaded
                .or_else(|| finished.then(|| d.total.map(|t| t.bytes)).flatten());
            Some(Progress::Part {
                part: choice.part_name(&d.format_id),
                finished,
                fraction: d.fraction(),
                bytes,
                done: bytes.map(units::bytes),
                total: d.total.map(|t| {
                    let words = units::bytes(t.bytes);
                    if t.approximate && !finished {
                        format!("~{words}")
                    } else {
                        words
                    }
                }),
                speed: d.speed.filter(|_| running).map(units::speed),
                left: d.eta.filter(|_| running).map(units::duration),
            })
        }
        Event::Postprocess {
            status: Status::Running,
            step,
        } => progress::describe_step(step).map(|words| Progress::Step { words }),
        _ => None,
    }
}

/// One update of the first run's installation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Install {
    pub tool: &'static str,
    /// "download", "check", "unpack" or "done".
    pub phase: &'static str,
    pub fraction: Option<f64>,
    pub done: Option<String>,
    pub total: Option<String>,
    pub speed: Option<String>,
}

/// When yt-dlp was last checked, `days` ago: "checked today".
pub fn checked(days: u64) -> String {
    match days {
        0 => "checked today".to_string(),
        1 => "checked yesterday".to_string(),
        n => format!("checked {n} days ago"),
    }
}

/// Why something failed, for the interface.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    /// What went wrong, in yt-dlp's words or keepclip's.
    pub said: String,
    /// The words are yt-dlp's.
    pub by_ytdlp: bool,
    /// The site seems to want a login.
    pub login: bool,
    /// Stopped on purpose, with Cancel: nothing to explain.
    pub stopped: bool,
}

impl From<keepclip_system::Error> for Failure {
    fn from(error: keepclip_system::Error) -> Failure {
        if let Some(failed) = error.downcast_ref::<Failed>() {
            return Failure {
                said: failed.said.clone(),
                by_ytdlp: true,
                login: failed.login,
                stopped: false,
            };
        }
        Failure {
            said: error.to_string(),
            by_ytdlp: false,
            login: false,
            stopped: error.is::<Stopped>(),
        }
    }
}

impl From<&str> for Failure {
    fn from(said: &str) -> Failure {
        Failure::from(keepclip_system::Error::from(said))
    }
}

impl From<String> for Failure {
    fn from(said: String) -> Failure {
        Failure::from(keepclip_system::Error::from(said))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keepclip_core::info::Size;
    use keepclip_core::progress::Download;

    const LIST: &str = r#"{
        "id": "list", "_type": "playlist", "uploader": "Sample uploader",
        "entries": [
            {"id": "a", "title": "Same caption", "duration": 14, "playlist_index": 1,
             "formats": [{"format_id": "v1", "vcodec": "avc1", "acodec": "mp4a", "height": 720, "width": 1280, "filesize": 1048576}]},
            null,
            {"id": "b", "title": "Same caption", "duration": 8, "playlist_index": 3,
             "formats": [{"format_id": "v2", "vcodec": "avc1", "acodec": "mp4a", "height": 720, "width": 1280}]}
        ]
    }"#;

    #[test]
    fn a_list_keeps_each_video_with_its_position() {
        let info = VideoInfo::parse(LIST).unwrap();
        let videos = videos(&info).unwrap();
        let items: Vec<Option<u64>> = videos.iter().map(|v| v.playlist_item).collect();
        assert_eq!(items, [Some(1), Some(3)]);
        let view = link(7, &info, &videos);
        assert_eq!(view.read, 7);
        assert_eq!(view.author.as_deref(), Some("Sample uploader"));
        assert_eq!(view.videos[1].position.as_deref(), Some("2 of 2"));
        assert_eq!(view.videos[1].duration.as_deref(), Some("0:08"));
        let first = &view.videos[0].choices[0];
        assert_eq!(
            (first.label.as_str(), first.size.as_str(), first.bytes),
            ("720p", "1.0 MB", Some(1_048_576))
        );
        assert_eq!(first.parts, ["Video"]);
        assert!(view.videos[1].choices[0].unknown);
    }

    #[test]
    fn a_link_with_nothing_to_download_is_an_error() {
        let info = VideoInfo::parse(r#"{"id": "x", "formats": []}"#).unwrap();
        assert!(videos(&info).is_err());
    }

    #[test]
    fn progress_lines_become_words() {
        let info = VideoInfo::parse(LIST).unwrap();
        let choice = &videos(&info).unwrap()[0].choices[0];
        let running = Event::Download(Download {
            status: Status::Running,
            format_id: "v1".to_string(),
            downloaded: Some(524_288),
            total: Some(Size {
                bytes: 1_048_576,
                approximate: true,
            }),
            speed: Some(262_144.0),
            eta: Some(2),
        });
        assert_eq!(
            progress(choice, &running),
            Some(Progress::Part {
                part: "Video",
                finished: false,
                fraction: Some(0.5),
                bytes: Some(524_288),
                done: Some("512 KB".to_string()),
                total: Some("~1.0 MB".to_string()),
                speed: Some("256 KB/s".to_string()),
                left: Some("0:02".to_string()),
            })
        );
        let joining = Event::Postprocess {
            status: Status::Running,
            step: "Merger".to_string(),
        };
        assert_eq!(
            progress(choice, &joining),
            Some(Progress::Step {
                words: "Joining video and sound"
            })
        );
        assert_eq!(progress(choice, &Event::Saved("x".to_string())), None);
    }

    #[test]
    fn failures_keep_the_words_of_yt_dlp() {
        let failure = Failure::from(keepclip_system::Error::from(Stopped));
        assert!(failure.stopped);
        let failure = Failure::from("Could not run yt-dlp");
        assert_eq!(failure.said, "Could not run yt-dlp");
        assert!(!failure.by_ytdlp && !failure.stopped);
    }

    #[test]
    fn the_last_check_in_words() {
        assert_eq!(checked(0), "checked today");
        assert_eq!(checked(1), "checked yesterday");
        assert_eq!(checked(9), "checked 9 days ago");
    }
}
