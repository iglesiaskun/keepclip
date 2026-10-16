//! The choices shown to the user: one per resolution, then sound only.
//!
//! Each resolution takes the video that plays almost everywhere when there is
//! one: H.264, joined with AAC sound in an MP4 file. Above the highest H.264
//! the site has, each resolution takes the best video yt-dlp lists there;
//! below it, resolutions without H.264 are left out.

use crate::info::{Format, Size, VideoInfo};
use crate::units;

/// Bitrate of the MP3 files, in kilobits per second. A fixed bitrate makes
/// the size known in advance: the length times the bitrate.
pub const MP3_KBPS: u64 = 192;

/// One line of the menu.
#[derive(Debug, Clone, PartialEq)]
pub enum Choice {
    Video(VideoChoice),
    /// Only the sound, converted to MP3.
    AudioMp3 {
        size: Option<Size>,
        /// The `-f` argument: the best sound, or the best video when it
        /// carries the sound itself. Sites with video and sound in one file
        /// may list a separate track that is not the video's sound, such as
        /// the whole song behind a short clip.
        source: String,
    },
}

/// `-f` for the best sound yt-dlp finds, or the best file when none has
/// sound alone.
pub const BEST_SOUND: &str = "ba/b";

/// A video at one resolution, with sound when the site has it.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoChoice {
    /// The number in "1080p"; `None` when the site does not say.
    pub resolution: Option<u64>,
    pub fps: Option<f64>,
    /// yt-dlp id of the video format, which may carry the sound already.
    pub video: String,
    /// Id of the separate sound format, when the video has none.
    pub audio: Option<String>,
    /// Video and sound together; `None` when the site does not give it.
    pub size: Option<Size>,
    /// yt-dlp's `--merge-output-format` for joining the two: `"mp4"` when both
    /// already come in MP4 files, which always fit together; otherwise
    /// `"mp4/mkv"`, MP4 when yt-dlp knows the codecs fit and MKV when not.
    pub merge_into: &'static str,
}

impl VideoChoice {
    /// The `-f` argument of yt-dlp for this choice.
    pub fn format_selector(&self) -> String {
        match &self.audio {
            Some(audio) => format!("{}+{audio}", self.video),
            None => self.video.clone(),
        }
    }
}

impl Choice {
    /// What the line says, without the size: "1080p", "720p 60 fps",
    /// "Audio only (MP3)".
    pub fn label(&self) -> String {
        match self {
            Choice::Video(v) => {
                let mut label = match v.resolution {
                    Some(lines) => format!("{lines}p"),
                    None => "Best available".to_string(),
                };
                // 50 fps and up is worth saying; 24, 25 and 30 are the usual.
                if let Some(fps) = v.fps.filter(|fps| *fps >= 49.0) {
                    label.push_str(&format!(" {} fps", fps.round()));
                }
                label
            }
            Choice::AudioMp3 { .. } => "Audio only (MP3)".to_string(),
        }
    }

    pub fn size(&self) -> Option<Size> {
        match self {
            Choice::Video(v) => v.size,
            Choice::AudioMp3 { size, .. } => *size,
        }
    }

    /// "48.2 MB", "~48.2 MB" for an estimate, or "size unknown".
    pub fn size_text(&self) -> String {
        match self.size() {
            Some(size) if size.approximate => format!("~{}", units::bytes(size.bytes)),
            Some(size) => units::bytes(size.bytes),
            None => "size unknown".to_string(),
        }
    }
}

/// The menu for one video: the resolutions from the highest down, then sound
/// only. Empty when yt-dlp found nothing to download.
pub fn build(info: &VideoInfo) -> Vec<Choice> {
    let formats = &info.formats;
    let audio = best_audio(formats);
    let mut videos: Vec<&Format> = formats.iter().filter(|f| f.has_video()).collect();
    // Without separate sound, a video with sound beats a silent one.
    if audio.is_none() && videos.iter().any(|f| f.has_audio()) {
        videos.retain(|f| f.has_audio());
    }

    let mut resolutions: Vec<u64> = videos.iter().filter_map(|f| f.resolution()).collect();
    resolutions.sort_unstable_by(|a, b| b.cmp(a));
    resolutions.dedup();
    // Below the highest H.264, a resolution without H.264 only adds a line
    // that plays in fewer places.
    let h264_at = |lines: u64| {
        videos
            .iter()
            .any(|f| f.is_h264() && f.resolution() == Some(lines))
    };
    let top_h264 = resolutions.iter().copied().find(|&lines| h264_at(lines));
    resolutions.retain(|&lines| Some(lines) > top_h264 || h264_at(lines));

    let mut menu: Vec<Choice> = resolutions
        .iter()
        .filter_map(|&lines| {
            let at_this_size: Vec<&Format> = videos
                .iter()
                .copied()
                .filter(|f| f.resolution() == Some(lines))
                .collect();
            best_video(&at_this_size)
                .map(|video| Choice::Video(video_choice(video, audio, info.duration)))
        })
        .collect();
    if menu.is_empty()
        && let Some(video) = best_video(&videos)
    {
        menu.push(Choice::Video(video_choice(video, audio, info.duration)));
    }

    if audio.is_some() || formats.iter().any(|f| f.has_audio()) {
        let size = info.duration.map(|seconds| Size {
            bytes: (seconds * (MP3_KBPS * 1000 / 8) as f64) as u64,
            approximate: true,
        });
        let source = match menu.first() {
            Some(Choice::Video(top)) if top.audio.is_none() => top.video.clone(),
            _ => BEST_SOUND.to_string(),
        };
        menu.push(Choice::AudioMp3 { size, source });
    }
    menu
}

fn video_choice(video: &Format, audio: Option<&Format>, duration: Option<f64>) -> VideoChoice {
    let audio = audio.filter(|_| !video.has_audio());
    let size = match audio {
        Some(audio) => video
            .size(duration)
            .zip(audio.size(duration))
            .map(|(v, a)| v + a),
        None => video.size(duration),
    };
    let in_mp4 = |f: &Format| matches!(f.ext.as_deref(), Some("mp4" | "m4a" | "m4v"));
    let merge_into = match audio {
        Some(audio) if in_mp4(video) && in_mp4(audio) => "mp4",
        _ => "mp4/mkv",
    };
    VideoChoice {
        resolution: video.resolution(),
        fps: video.fps,
        video: video.format_id.clone(),
        audio: audio.map(|a| a.format_id.clone()),
        size,
        merge_into,
    }
}

/// The last H.264 video, or the last video when there is no H.264.
fn best_video<'a>(videos: &[&'a Format]) -> Option<&'a Format> {
    let mut from_best = videos.iter().rev().copied();
    from_best
        .clone()
        .find(|f| f.is_h264())
        .or_else(|| from_best.next())
}

/// The last AAC sound, or the last sound when there is no AAC.
fn best_audio(formats: &[Format]) -> Option<&Format> {
    let mut from_best = formats.iter().rev().filter(|f| f.is_audio_only());
    from_best
        .clone()
        .find(|f| f.is_aac())
        .or_else(|| from_best.next())
}
