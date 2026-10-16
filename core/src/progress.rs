//! Follows a download through the lines yt-dlp prints.
//!
//! keepclip asks yt-dlp for one line per update (`--newline`), in templates of
//! its own, each starting with a word no other line starts with. Fields
//! yt-dlp does not know come as `NA`.

use crate::info::Size;

const DOWNLOAD: &str = "keepclip-progress";
const POSTPROCESS: &str = "keepclip-postprocess";
const SAVED: &str = "keepclip-file";

/// `--progress-template` for the download. The format id goes last: it is
/// the only field that might contain a space.
pub fn download_template() -> String {
    format!(
        "download:{DOWNLOAD} %(progress.status)s %(progress.downloaded_bytes)s \
         %(progress.total_bytes)s %(progress.total_bytes_estimate)s %(progress.speed)s \
         %(progress.eta)s %(info.format_id)s"
    )
}

/// `--progress-template` for the steps after the download (joining video and
/// sound, converting to MP3).
pub fn postprocess_template() -> String {
    format!("postprocess:{POSTPROCESS} %(progress.status)s %(progress.postprocessor)s")
}

/// `--print` for the path of the finished file, once it is in its folder.
pub fn saved_template() -> String {
    format!("after_move:{SAVED} %(filepath)s")
}

/// What one line says.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Download(Download),
    /// A step after the download, such as joining video and sound.
    Postprocess {
        status: Status,
        step: String,
    },
    /// The finished file, in its folder.
    Saved(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Downloading, or for a step, running.
    Running,
    Finished,
    Error,
}

/// Progress of one file. A video with separate sound downloads two files, one
/// after the other.
#[derive(Debug, Clone, PartialEq)]
pub struct Download {
    pub status: Status,
    pub format_id: String,
    pub downloaded: Option<u64>,
    /// Exact, or estimated while a stream in pieces is downloading.
    pub total: Option<Size>,
    /// Bytes per second.
    pub speed: Option<f64>,
    /// Seconds left.
    pub eta: Option<u64>,
}

impl Download {
    /// Done so far, from 0 to 1.
    pub fn fraction(&self) -> Option<f64> {
        if self.status == Status::Finished {
            return Some(1.0);
        }
        let total = self.total.filter(|t| t.bytes > 0)?.bytes;
        Some((self.downloaded? as f64 / total as f64).clamp(0.0, 1.0))
    }
}

/// Reads one line of yt-dlp's output. `None` for lines that are not keepclip's
/// templates: messages, warnings, errors.
pub fn parse(line: &str) -> Option<Event> {
    let line = line.trim_end_matches(['\r', '\n']);
    let (word, rest) = line.split_once(' ')?;
    match word {
        DOWNLOAD => parse_download(rest).map(Event::Download),
        POSTPROCESS => {
            let (status, step) = rest.split_once(' ')?;
            Some(Event::Postprocess {
                status: status_of(status)?,
                step: step.to_string(),
            })
        }
        SAVED if !rest.is_empty() => Some(Event::Saved(rest.to_string())),
        _ => None,
    }
}

fn parse_download(fields: &str) -> Option<Download> {
    let mut parts = fields.splitn(7, ' ');
    let status = status_of(parts.next()?)?;
    let downloaded = whole(parts.next()?);
    let total = whole(parts.next()?);
    let estimate = whole(parts.next()?);
    let speed = number(parts.next()?);
    let eta = whole(parts.next()?);
    let format_id = parts.next()?.to_string();
    let total = match (total, estimate) {
        (Some(bytes), _) => Some(Size {
            bytes,
            approximate: false,
        }),
        (None, Some(bytes)) => Some(Size {
            bytes,
            approximate: true,
        }),
        (None, None) => None,
    };
    Some(Download {
        status,
        format_id,
        downloaded,
        total,
        speed,
        eta,
    })
}

fn status_of(word: &str) -> Option<Status> {
    match word {
        "downloading" | "started" | "processing" => Some(Status::Running),
        "finished" => Some(Status::Finished),
        "error" => Some(Status::Error),
        _ => None,
    }
}

/// `NA` and anything else that is not a number count as unknown.
fn number(field: &str) -> Option<f64> {
    field
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && *n >= 0.0)
}

fn whole(field: &str) -> Option<u64> {
    number(field).map(|n| n as u64)
}

/// What a step after the download is doing, in words, or `None` for steps
/// not worth showing (moving the file to its folder).
pub fn describe_step(step: &str) -> Option<&'static str> {
    match step {
        "Merger" => Some("Joining video and sound"),
        "ExtractAudio" => Some("Converting to MP3"),
        step if step.starts_with("Fixup") => Some("Fixing the file"),
        _ => None,
    }
}
