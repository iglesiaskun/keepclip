//! Downloads over HTTPS, with the SHA-256 computed on the way, and unzips.

use crate::Result;
use crate::console::{self, StatusLine};
use keepclip_core::units;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .user_agent(concat!("keepclip/", env!("CARGO_PKG_VERSION")))
            .timeout_connect(Some(Duration::from_secs(30)))
            .timeout_recv_response(Some(Duration::from_secs(60)))
            .build()
            .new_agent()
    })
}

/// A small text file, such as a list of checksums.
pub fn text(url: &str) -> Result<String> {
    let mut response = agent()
        .get(url)
        .call()
        .map_err(|e| format!("Could not download {url}: {e}"))?;
    Ok(response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Could not download {url}: {e}"))?)
}

/// Downloads `url` into `destination`, showing the progress after `label`,
/// and returns the SHA-256 of what was written, in lowercase hex.
pub fn file(url: &str, destination: &Path, label: &str) -> Result<String> {
    let fail = |e: &dyn std::fmt::Display| format!("Could not download {url}: {e}");
    let mut response = agent().get(url).call().map_err(|e| fail(&e))?;
    let total = response.body().content_length();
    let mut reader = response.body_mut().as_reader();
    let mut writer = BufWriter::new(File::create(destination).map_err(|e| fail(&e))?);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 256 * 1024];
    let mut done: u64 = 0;
    let mut line = StatusLine::default();
    let show = |line: &mut StatusLine, done: u64| {
        let fraction = total.map(|t| done as f64 / t.max(1) as f64);
        let mut text = format!("{label:<7} {}", console::bar(fraction));
        text.push_str(&format!(" {}", units::bytes(done)));
        if let Some(total) = total {
            text.push_str(&format!(" of {}", units::bytes(total)));
        }
        line.show(&text);
    };
    loop {
        let read = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(fail(&e).into()),
        };
        hasher.update(&buffer[..read]);
        writer.write_all(&buffer[..read]).map_err(|e| fail(&e))?;
        let before = done;
        done += read as u64;
        // Redraw once per megabyte, not on every read.
        if before / (1024 * 1024) != done / (1024 * 1024) {
            show(&mut line, done);
        }
    }
    writer.flush().map_err(|e| fail(&e))?;
    show(&mut line, done);
    line.finish();
    let hash = hasher.finalize();
    Ok(hash.iter().map(|b| format!("{b:02x}")).collect())
}

/// Extracts the files of `archive` for which `pick` gives a path, relative
/// to `destination`. Returns how many were extracted.
pub fn unzip(
    archive: &Path,
    destination: &Path,
    pick: impl Fn(&Path) -> Option<PathBuf>,
) -> Result<usize> {
    let fail = |e: &dyn std::fmt::Display| format!("Could not unzip {}: {e}", archive.display());
    let mut zip =
        zip::ZipArchive::new(File::open(archive).map_err(|e| fail(&e))?).map_err(|e| fail(&e))?;
    let mut extracted = 0;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(|e| fail(&e))?;
        // `enclosed_name` refuses names that would leave the folder ("../").
        let Some(inside) = entry.enclosed_name() else {
            continue;
        };
        let Some(relative) = entry.is_file().then(|| pick(&inside)).flatten() else {
            continue;
        };
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| fail(&e))?;
        }
        let mut out = BufWriter::new(File::create(&target).map_err(|e| fail(&e))?);
        io::copy(&mut entry, &mut out).map_err(|e| fail(&e))?;
        out.flush().map_err(|e| fail(&e))?;
        extracted += 1;
    }
    Ok(extracted)
}
