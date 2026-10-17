//! The programs keepclip runs: yt-dlp, and ffmpeg and Deno, which yt-dlp
//! needs. On Windows, keepclip downloads them on the first run into
//! `%LOCALAPPDATA%\keepclip\bin`, from their official releases, and checks
//! each against the SHA-256 published with it. Elsewhere, they must be in
//! the PATH.
//!
//! Asking the user is left to each program: this module only says what is
//! missing and installs what it is told to.

use crate::Result;
use crate::fetch;
use crate::stop::Stop;
use crate::ytdlp::YtDlp;
use keepclip_core::checksum;
use keepclip_core::download::Tools;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

/// yt-dlp follows the sites, which change often: an old copy stops
/// working. keepclip updates it when the last check is older than this.
const UPDATE_EVERY: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// File whose date is the last time yt-dlp was installed or updated.
const UPDATE_STAMP: &str = "yt-dlp.checked";

pub struct Tool {
    pub name: &'static str,
    /// What it does, for the question on the first run.
    pub role: &'static str,
    /// Where it comes from, for the same question.
    pub source: &'static str,
    /// Rough download size, for the same question.
    pub megabytes: u32,
    url: &'static str,
    /// The published checksums, and the name the file has in them.
    sums_url: &'static str,
    file_name: &'static str,
    /// The file that shows the tool is installed, relative to `bin`.
    installed: &'static str,
    /// The argument that makes it print its version on the first line.
    version_arg: &'static str,
    unpack: Unpack,
}

enum Unpack {
    /// The download is the program itself.
    Executable,
    /// A zip; `pick` says where each file goes, relative to `bin`.
    Zip(fn(&Path) -> Option<PathBuf>),
}

const TOOLS: [Tool; 3] = [
    Tool {
        name: "yt-dlp",
        role: "downloads the videos",
        source: "github.com/yt-dlp/yt-dlp",
        megabytes: 18,
        url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
        sums_url: "https://github.com/yt-dlp/yt-dlp/releases/latest/download/SHA2-256SUMS",
        file_name: "yt-dlp.exe",
        installed: "yt-dlp.exe",
        version_arg: "--version",
        unpack: Unpack::Executable,
    },
    Tool {
        name: "ffmpeg",
        role: "joins video and sound, makes MP3s",
        // The builds the yt-dlp project makes for it, with its fixes.
        source: "github.com/yt-dlp/FFmpeg-Builds",
        megabytes: 90,
        url: "https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl-shared.zip",
        sums_url: "https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/checksums.sha256",
        file_name: "ffmpeg-master-latest-win64-gpl-shared.zip",
        installed: "ffmpeg\\ffmpeg.exe",
        version_arg: "-version",
        // The programs and their DLLs are in `<top folder>\bin`. yt-dlp
        // uses ffmpeg and ffprobe; the player that comes with them is
        // left out.
        unpack: Unpack::Zip(|path| {
            let mut parts = path.iter();
            let (_top, bin, file) = (parts.next()?, parts.next()?, parts.next()?);
            let name = file.to_string_lossy().to_ascii_lowercase();
            let wanted = name == "ffmpeg.exe" || name == "ffprobe.exe" || name.ends_with(".dll");
            (bin == "bin" && parts.next().is_none() && wanted)
                .then(|| Path::new("ffmpeg").join(file))
        }),
    },
    Tool {
        name: "Deno",
        role: "runs the JavaScript some sites need",
        source: "github.com/denoland/deno",
        megabytes: 43,
        url: "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip",
        sums_url: "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip.sha256sum",
        file_name: "deno-x86_64-pc-windows-msvc.zip",
        installed: "deno.exe",
        version_arg: "--version",
        unpack: Unpack::Zip(|path| (path == Path::new("deno.exe")).then(|| path.to_path_buf())),
    },
];

/// The programs keepclip downloads. None outside Windows, where they come
/// from the system's package manager, in the PATH.
pub fn downloaded() -> &'static [Tool] {
    if cfg!(windows) { &TOOLS } else { &[] }
}

/// Where the downloaded programs go: `%LOCALAPPDATA%\keepclip\bin`.
pub fn bin_folder() -> Result<PathBuf> {
    Ok(dirs::data_local_dir()
        .ok_or("Could not find the local app data folder.")?
        .join("keepclip")
        .join("bin"))
}

/// The programs that are not installed yet.
pub fn missing(bin: &Path) -> Vec<&'static Tool> {
    downloaded()
        .iter()
        .filter(|tool| !bin.join(tool.installed).is_file())
        .collect()
}

/// yt-dlp, with ffmpeg and Deno next to it: from `bin` on Windows, from the
/// PATH elsewhere.
pub fn ytdlp(bin: &Path) -> YtDlp {
    if downloaded().is_empty() {
        return YtDlp::new("yt-dlp".into(), Tools::default());
    }
    YtDlp::new(
        bin.join("yt-dlp.exe"),
        Tools {
            ffmpeg: Some(bin.join("ffmpeg")),
            deno: Some(bin.join("deno.exe")),
        },
    )
}

/// A step of [`install`], for its progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Bytes downloaded so far, and the total when the server says it.
    Download { done: u64, total: Option<u64> },
    /// Checking the download against the published checksum.
    Check,
    /// Unzipping.
    Unpack,
}

/// Downloads and installs one program into `bin`, calling `on_step` as it
/// goes. `stop` ends it early, leaving no half-installed program.
pub fn install(bin: &Path, tool: &Tool, on_step: &mut dyn FnMut(Step), stop: &Stop) -> Result<()> {
    fs::create_dir_all(bin).map_err(|e| format!("Could not create {}: {e}", bin.display()))?;
    let sums = fetch::text(tool.sums_url)?;
    let expected = checksum::find_sha256(&sums, tool.file_name).ok_or_else(|| {
        format!(
            "The checksums of {} do not list {}.",
            tool.name, tool.file_name
        )
    })?;
    let download = bin.join(format!("{}.download", tool.file_name));
    let actual = fetch::file(
        tool.url,
        &download,
        &mut |done, total| on_step(Step::Download { done, total }),
        stop,
    )?;
    on_step(Step::Check);
    if actual != expected {
        let _ = fs::remove_file(&download);
        return Err(format!(
            "The download of {} does not match its published checksum, so keepclip \
             deleted it. Try again later; if it keeps happening, please report it.",
            tool.name
        )
        .into());
    }
    let result = match tool.unpack {
        Unpack::Executable => fs::rename(&download, bin.join(tool.installed))
            .map_err(|e| format!("Could not install {}: {e}", tool.name).into()),
        Unpack::Zip(pick) => {
            on_step(Step::Unpack);
            unzip(bin, tool, &download, pick)
        }
    };
    let _ = fs::remove_file(&download);
    if result.is_ok() && tool.name == "yt-dlp" {
        stamp(&bin.join(UPDATE_STAMP));
    }
    result
}

/// Unzips into a new folder first, so a failure leaves no half tool.
fn unzip(bin: &Path, tool: &Tool, zip: &Path, pick: fn(&Path) -> Option<PathBuf>) -> Result<()> {
    let staging = bin.join(format!("{}.new", tool.file_name));
    let _ = fs::remove_dir_all(&staging);
    let count = fetch::unzip(zip, &staging, pick)?;
    if count == 0 || !staging.join(tool.installed).is_file() {
        let _ = fs::remove_dir_all(&staging);
        return Err(format!(
            "The download of {} does not have the expected files.",
            tool.name
        )
        .into());
    }
    for entry in fs::read_dir(&staging)? {
        let entry = entry?;
        let target = bin.join(entry.file_name());
        if target.is_dir() {
            fs::remove_dir_all(&target)?;
        }
        fs::rename(entry.path(), &target)
            .map_err(|e| format!("Could not install {}: {e}", tool.name))?;
    }
    let _ = fs::remove_dir_all(&staging);
    Ok(())
}

/// When yt-dlp was last installed or updated.
pub fn last_check(bin: &Path) -> Option<SystemTime> {
    fs::metadata(bin.join(UPDATE_STAMP))
        .and_then(|m| m.modified())
        .ok()
}

/// Whether yt-dlp is due for its weekly update.
pub fn update_due(bin: &Path) -> bool {
    if downloaded().is_empty() {
        return false;
    }
    match last_check(bin).map(|time| SystemTime::now().duration_since(time)) {
        Some(Ok(age)) => age > UPDATE_EVERY,
        // A date in the future: the clock changed. Check now.
        Some(Err(_)) => true,
        None => true,
    }
}

fn stamp(path: &Path) {
    // Without the stamp, keepclip only checks for updates on every run.
    let _ = fs::write(
        path,
        "The date of this file is the last update check of yt-dlp.\n",
    );
}

/// `yt-dlp -U` replaces yt-dlp with its latest release, after checking it.
/// Returns its last line, or why it failed: then the installed copy may
/// still work.
///
/// It runs outside any job: closing keepclip during the update lets it
/// finish, so yt-dlp is never left half replaced.
pub fn update(bin: &Path) -> std::result::Result<String, String> {
    let mut command = Command::new(bin.join("yt-dlp.exe"));
    command.arg("-U").stdin(Stdio::null());
    crate::no_console(&mut command);
    let last_line = |bytes: &[u8]| {
        String::from_utf8_lossy(bytes)
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string()
    };
    match command.output() {
        Ok(output) if output.status.success() => {
            stamp(&bin.join(UPDATE_STAMP));
            Ok(last_line(&output.stdout))
        }
        Ok(output) => Err(last_line(&output.stderr)),
        Err(e) => Err(e.to_string()),
    }
}

/// The version of each downloaded program, as it reports it, or `None`
/// when it is missing or does not say.
pub fn versions(bin: &Path) -> Vec<(&'static str, Option<String>)> {
    downloaded()
        .iter()
        .map(|tool| (tool.name, version(bin, tool)))
        .collect()
}

fn version(bin: &Path, tool: &Tool) -> Option<String> {
    if tool.name == "yt-dlp" {
        return ytdlp(bin).version();
    }
    let mut command = Command::new(bin.join(tool.installed));
    command
        .arg(tool.version_arg)
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    crate::no_console(&mut command);
    let output = command.output().ok().filter(|o| o.status.success())?;
    let text = String::from_utf8_lossy(&output.stdout);
    version_in(tool.name, text.lines().next()?)
}

/// The version in the first line a program prints: "ffmpeg version
/// N-121234-gabc-20261001 Copyright..." gives "N-121234-gabc-20261001",
/// "deno 2.9.7 (stable, ...)" gives "2.9.7".
fn version_in(name: &str, line: &str) -> Option<String> {
    let mut words = line.split_whitespace();
    let first = words.next()?;
    if !first.eq_ignore_ascii_case(name) {
        return None;
    }
    let mut next = words.next()?;
    if next == "version" {
        next = words.next()?;
    }
    Some(next.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_come_from_the_first_line() {
        assert_eq!(
            version_in(
                "ffmpeg",
                "ffmpeg version N-121234-gabc-20261001 Copyright (c) 2000-2026"
            )
            .as_deref(),
            Some("N-121234-gabc-20261001")
        );
        assert_eq!(
            version_in(
                "Deno",
                "deno 2.9.7 (stable, release, x86_64-pc-windows-msvc)"
            )
            .as_deref(),
            Some("2.9.7")
        );
        assert_eq!(version_in("Deno", "something else"), None);
    }

    #[test]
    fn the_ffmpeg_zip_gives_only_its_programs() {
        let Unpack::Zip(pick) = TOOLS[1].unpack else {
            panic!("ffmpeg comes zipped");
        };
        let top = "ffmpeg-master-latest-win64-gpl-shared";
        assert_eq!(
            pick(&Path::new(top).join("bin").join("ffmpeg.exe")),
            Some(Path::new("ffmpeg").join("ffmpeg.exe"))
        );
        assert_eq!(
            pick(&Path::new(top).join("bin").join("avcodec-62.dll")),
            Some(Path::new("ffmpeg").join("avcodec-62.dll"))
        );
        assert_eq!(pick(&Path::new(top).join("bin").join("ffplay.exe")), None);
        assert_eq!(pick(&Path::new(top).join("doc").join("ffmpeg.html")), None);
    }
}
