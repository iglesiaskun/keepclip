//! The programs keepclip runs: yt-dlp, and ffmpeg and Deno, which yt-dlp
//! needs. On Windows, keepclip downloads them on the first run into
//! `%LOCALAPPDATA%\keepclip\bin`, from their official releases, and checks
//! each against the SHA-256 published with it. Elsewhere, they must be in
//! the PATH.

use crate::Result;
use crate::ytdlp::YtDlp;

#[cfg(windows)]
pub use windows_tools::ready;

/// Outside Windows, the programs come from the PATH, installed with the
/// system's package manager.
#[cfg(not(windows))]
pub fn ready() -> Result<YtDlp> {
    Ok(YtDlp::new(
        "yt-dlp".into(),
        keepclip_core::download::Tools::default(),
    ))
}

#[cfg(windows)]
mod windows_tools {
    use super::*;
    use crate::{console, fetch};
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

    struct Tool {
        name: &'static str,
        /// What it does, for the question on the first run.
        role: &'static str,
        /// Where it comes from, for the same question.
        source: &'static str,
        /// Rough download size, for the same question.
        megabytes: u32,
        url: &'static str,
        /// The published checksums, and the name the file has in them.
        sums_url: &'static str,
        file_name: &'static str,
        /// The file that shows the tool is installed, relative to `bin`.
        installed: &'static str,
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
            // The programs and their DLLs are in `<top folder>\bin`. yt-dlp
            // uses ffmpeg and ffprobe; the player that comes with them is
            // left out.
            unpack: Unpack::Zip(|path| {
                let mut parts = path.iter();
                let (_top, bin, file) = (parts.next()?, parts.next()?, parts.next()?);
                let name = file.to_string_lossy().to_ascii_lowercase();
                let wanted =
                    name == "ffmpeg.exe" || name == "ffprobe.exe" || name.ends_with(".dll");
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
            unpack: Unpack::Zip(|path| (path == Path::new("deno.exe")).then(|| path.to_path_buf())),
        },
    ];

    /// yt-dlp, installed and recent, with ffmpeg and Deno next to it.
    pub fn ready() -> Result<YtDlp> {
        let bin = dirs::data_local_dir()
            .ok_or("Could not find the local app data folder.")?
            .join("keepclip")
            .join("bin");
        let missing: Vec<&Tool> = TOOLS
            .iter()
            .filter(|tool| !bin.join(tool.installed).is_file())
            .collect();
        if !missing.is_empty() {
            install(&bin, &missing)?;
        }
        let program = bin.join("yt-dlp.exe");
        if is_stale(&bin.join(UPDATE_STAMP)) {
            update(&program, &bin);
        }
        Ok(YtDlp::new(
            program,
            Tools {
                ffmpeg: Some(bin.join("ffmpeg")),
                deno: Some(bin.join("deno.exe")),
            },
        ))
    }

    fn install(bin: &Path, missing: &[&Tool]) -> Result<()> {
        println!(
            "\nkeepclip needs these free programs, downloaded once into {}:",
            bin.display()
        );
        for tool in missing {
            println!(
                "  {:<7} {} (about {} MB, from {})",
                tool.name, tool.role, tool.megabytes, tool.source
            );
        }
        let answer = console::ask("Download them now? (Y/n) ")?;
        if answer.eq_ignore_ascii_case("n") || answer.eq_ignore_ascii_case("no") {
            return Err("keepclip cannot download videos without them.".into());
        }
        fs::create_dir_all(bin).map_err(|e| format!("Could not create {}: {e}", bin.display()))?;
        for tool in missing {
            install_one(bin, tool)?;
        }
        println!("All set.");
        Ok(())
    }

    fn install_one(bin: &Path, tool: &Tool) -> Result<()> {
        let sums = fetch::text(tool.sums_url)?;
        let expected = checksum::find_sha256(&sums, tool.file_name).ok_or_else(|| {
            format!(
                "The checksums of {} do not list {}.",
                tool.name, tool.file_name
            )
        })?;
        let download = bin.join(format!("{}.download", tool.file_name));
        let actual = fetch::file(tool.url, &download, tool.name)?;
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
            Unpack::Zip(pick) => unzip(bin, tool, &download, pick),
        };
        let _ = fs::remove_file(&download);
        if result.is_ok() && tool.name == "yt-dlp" {
            stamp(&bin.join(UPDATE_STAMP));
        }
        result
    }

    /// Unzips into a new folder first, so a failure leaves no half tool.
    fn unzip(
        bin: &Path,
        tool: &Tool,
        zip: &Path,
        pick: fn(&Path) -> Option<PathBuf>,
    ) -> Result<()> {
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

    fn is_stale(stamp: &Path) -> bool {
        let checked = fs::metadata(stamp).and_then(|m| m.modified());
        match checked.map(|time| SystemTime::now().duration_since(time)) {
            Ok(Ok(age)) => age > UPDATE_EVERY,
            // A date in the future: the clock changed. Check now.
            Ok(Err(_)) => true,
            Err(_) => true,
        }
    }

    fn stamp(path: &Path) {
        // Without the stamp, keepclip only checks for updates on every run.
        let _ = fs::write(
            path,
            "The date of this file is the last update check of yt-dlp.\n",
        );
    }

    /// `yt-dlp -U` replaces yt-dlp with its latest release, after checking
    /// it. A failure is only a warning: the installed copy may still work.
    fn update(program: &Path, bin: &Path) {
        println!("Checking for a newer yt-dlp...");
        let output = Command::new(program)
            .arg("-U")
            .stdin(Stdio::null())
            .output();
        match output {
            Ok(output) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = text.lines().rev().find(|l| !l.trim().is_empty()) {
                    println!("{}", line.trim());
                }
                stamp(&bin.join(UPDATE_STAMP));
            }
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stderr);
                let reason = text
                    .lines()
                    .rev()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("");
                println!(
                    "Could not update yt-dlp; going on with this one. {}",
                    reason.trim()
                );
            }
            Err(e) => println!("Could not update yt-dlp; going on with this one. {e}"),
        }
    }
}
