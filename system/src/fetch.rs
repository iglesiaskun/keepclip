//! Downloads over HTTPS, with the SHA-256 computed on the way, and unzips.

use crate::Result;
use crate::stop::Stop;
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

/// A small file, read into memory: at most `limit` bytes, or an error.
pub fn bytes(url: &str, limit: u64) -> Result<Vec<u8>> {
    let fail = |e: &dyn std::fmt::Display| format!("Could not download {url}: {e}");
    let mut response = agent().get(url).call().map_err(|e| fail(&e))?;
    Ok(response
        .body_mut()
        .with_config()
        .limit(limit)
        .read_to_vec()
        .map_err(|e| fail(&e))?)
}

/// Downloads `url` into `destination`, calling `on_progress` with the bytes
/// written so far and the total, when the server says it, and returns the
/// SHA-256 of what was written, in lowercase hex. `stop` ends it early with
/// [`crate::stop::Stopped`], and the partial file is deleted.
pub fn file(
    url: &str,
    destination: &Path,
    on_progress: &mut dyn FnMut(u64, Option<u64>),
    stop: &Stop,
) -> Result<String> {
    let fail = |e: &dyn std::fmt::Display| format!("Could not download {url}: {e}");
    let mut response = agent().get(url).call().map_err(|e| fail(&e))?;
    let total = response.body().content_length();
    let mut reader = response.body_mut().as_reader();
    let mut writer = BufWriter::new(File::create(destination).map_err(|e| fail(&e))?);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 256 * 1024];
    let mut done: u64 = 0;
    on_progress(done, total);
    loop {
        if let Err(stopped) = stop.check() {
            drop(writer);
            let _ = fs::remove_file(destination);
            return Err(stopped.into());
        }
        let read = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(fail(&e).into()),
        };
        hasher.update(&buffer[..read]);
        writer.write_all(&buffer[..read]).map_err(|e| fail(&e))?;
        done += read as u64;
        on_progress(done, total);
    }
    writer.flush().map_err(|e| fail(&e))?;
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
