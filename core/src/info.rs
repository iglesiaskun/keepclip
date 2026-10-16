//! What `yt-dlp -J` prints about a link, reduced to the fields keepclip uses.
//!
//! yt-dlp lists the formats from worst to best, so among formats that keepclip
//! considers equal, the last one wins.

use serde::{Deserialize, Deserializer};
use std::fmt;

/// One video, or a link with several of them (a playlist, a post with more
/// than one video).
#[derive(Debug, Clone, Deserialize)]
pub struct VideoInfo {
    #[serde(deserialize_with = "text")]
    pub id: String,
    #[serde(default, deserialize_with = "words")]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "words")]
    pub uploader: Option<String>,
    /// The account or channel; some sites give it where `uploader` is empty.
    #[serde(default, deserialize_with = "words")]
    pub channel: Option<String>,
    /// Length in seconds.
    #[serde(default, deserialize_with = "number")]
    pub duration: Option<f64>,
    /// `"playlist"` for a link with several videos; usually absent otherwise.
    #[serde(default, rename = "_type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub formats: Vec<Format>,
    /// The videos of a playlist. Entries the site hides come as `null`.
    #[serde(default)]
    pub entries: Vec<Option<VideoInfo>>,
    /// Position in the playlist, counting from 1, for an entry of one.
    #[serde(default, deserialize_with = "whole")]
    pub playlist_index: Option<u64>,
}

/// A file yt-dlp can download for a video: video, sound or both.
#[derive(Debug, Clone, Deserialize)]
pub struct Format {
    #[serde(deserialize_with = "text")]
    pub format_id: String,
    #[serde(default)]
    pub ext: Option<String>,
    /// `"none"` when the format has no video; absent when the site does not say.
    #[serde(default)]
    pub vcodec: Option<String>,
    /// `"none"` when the format has no sound; absent when the site does not say.
    #[serde(default)]
    pub acodec: Option<String>,
    #[serde(default, deserialize_with = "whole")]
    pub width: Option<u64>,
    #[serde(default, deserialize_with = "whole")]
    pub height: Option<u64>,
    #[serde(default, deserialize_with = "number")]
    pub fps: Option<f64>,
    /// Exact size in bytes.
    #[serde(default, deserialize_with = "whole")]
    pub filesize: Option<u64>,
    /// Size in bytes estimated from the bitrate and the length.
    #[serde(default, deserialize_with = "whole")]
    pub filesize_approx: Option<u64>,
    /// Average bitrate, video and sound together, in kilobits per second.
    #[serde(default, deserialize_with = "number")]
    pub tbr: Option<f64>,
    #[serde(default)]
    pub format_note: Option<String>,
}

/// Why the output of `yt-dlp -J` could not be read.
#[derive(Debug)]
pub struct ParseError(serde_json::Error);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unexpected video information from yt-dlp: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

impl VideoInfo {
    /// Reads the JSON that `yt-dlp -J` prints.
    pub fn parse(json: &str) -> Result<VideoInfo, ParseError> {
        serde_json::from_str(json).map_err(ParseError)
    }

    /// Whether the link holds several videos instead of one.
    pub fn is_playlist(&self) -> bool {
        self.kind.as_deref() == Some("playlist")
    }

    /// The videos of a playlist that can be downloaded.
    pub fn videos(&self) -> impl Iterator<Item = &VideoInfo> {
        self.entries.iter().flatten()
    }

    /// Who posted the video: the uploader, or the channel when the site
    /// leaves the uploader empty.
    pub fn author(&self) -> Option<&str> {
        self.uploader.as_deref().or(self.channel.as_deref())
    }
}

impl Format {
    /// Storyboards are grids of thumbnails, not videos.
    fn is_storyboard(&self) -> bool {
        self.ext.as_deref() == Some("mhtml") || self.format_note.as_deref() == Some("storyboard")
    }

    /// Whether the format has video. Unknown counts as yes, as in yt-dlp.
    pub fn has_video(&self) -> bool {
        self.vcodec.as_deref() != Some("none") && !self.is_storyboard()
    }

    /// Whether the format has sound. Unknown counts as yes, as in yt-dlp.
    pub fn has_audio(&self) -> bool {
        self.acodec.as_deref() != Some("none") && !self.is_storyboard()
    }

    /// Sound only.
    pub fn is_audio_only(&self) -> bool {
        self.has_audio() && self.vcodec.as_deref() == Some("none")
    }

    /// Lines of the shorter side, the number in "1080p". A vertical video of
    /// 1080 × 1920 is 1080p too.
    pub fn resolution(&self) -> Option<u64> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => Some(w.min(h)),
            (None, Some(h)) => Some(h),
            _ => None,
        }
    }

    /// H.264, the video codec that plays everywhere.
    pub fn is_h264(&self) -> bool {
        codec_starts_with(self.vcodec.as_deref(), &["avc1", "avc3", "h264"])
    }

    /// AAC, the sound codec that goes with H.264 in an MP4 file.
    pub fn is_aac(&self) -> bool {
        codec_starts_with(self.acodec.as_deref(), &["mp4a", "aac"])
    }

    /// The size, exact when the site gives it, estimated otherwise: by the
    /// site, or from the bitrate and the length (`duration`, in seconds).
    pub fn size(&self, duration: Option<f64>) -> Option<Size> {
        let estimate = || {
            self.filesize_approx.or_else(|| {
                let bytes_per_second = self.tbr? * 1000.0 / 8.0;
                Some((bytes_per_second * duration?) as u64)
            })
        };
        match self.filesize {
            Some(bytes) => Some(Size {
                bytes,
                approximate: false,
            }),
            None => estimate().map(|bytes| Size {
                bytes,
                approximate: true,
            }),
        }
    }
}

fn codec_starts_with(codec: Option<&str>, prefixes: &[&str]) -> bool {
    codec.is_some_and(|codec| {
        let codec = codec.to_ascii_lowercase();
        prefixes.iter().any(|p| codec.starts_with(p))
    })
}

/// A size in bytes, and whether it is an estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub bytes: u64,
    pub approximate: bool,
}

impl std::ops::Add for Size {
    type Output = Size;

    fn add(self, other: Size) -> Size {
        Size {
            bytes: self.bytes + other.bytes,
            approximate: self.approximate || other.approximate,
        }
    }
}

// Sites do not always use the JSON type one expects: an id may be a number,
// a size may have decimals. These readers accept both.

fn text<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    match serde_json::Value::deserialize(d)? {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        other => Err(serde::de::Error::custom(format!(
            "expected text, found {other}"
        ))),
    }
}

/// Text that is empty or only spaces counts as missing.
fn words<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(match Option::<serde_json::Value>::deserialize(d)? {
        Some(serde_json::Value::String(s)) if !s.trim().is_empty() => Some(s),
        _ => None,
    })
}

fn number<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    Ok(Option::<serde_json::Value>::deserialize(d)?
        .and_then(|v| v.as_f64())
        .filter(|n| n.is_finite() && *n >= 0.0))
}

fn whole<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    // The cast rounds toward zero and saturates; the filter keeps it in range.
    Ok(number(d)?.map(|n| n as u64))
}
