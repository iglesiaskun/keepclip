//! The work folder, where yt-dlp makes each file, and moving the finished
//! file into the download folder.

use crate::Result;
use std::path::{Path, PathBuf};

/// Where yt-dlp makes the files. Parts of an interrupted download stay here,
/// and yt-dlp resumes them when the same choice is downloaded again.
pub fn work_folder() -> PathBuf {
    std::env::temp_dir().join("keepclip")
}

/// What [`move_into`] did.
#[derive(Debug, PartialEq, Eq)]
pub enum Moved {
    /// The file is in the download folder, at this path.
    Saved(PathBuf),
    /// The folder already has a file with this name and replacing was not
    /// allowed: nothing moved. Ask, then move again or delete the new file.
    Exists(PathBuf),
}

/// Moves the finished file from the work folder into `folder`. A file with
/// the same name there is replaced only when `replace`, and only once the
/// new one is fully in place.
pub fn move_into(file: &Path, folder: &Path, replace: bool) -> Result<Moved> {
    let name = file
        .file_name()
        .ok_or("yt-dlp gave a path without a file name.")?;
    let target = folder.join(name);
    if target.exists() && !replace {
        return Ok(Moved::Exists(target));
    }
    let fail = |e: std::io::Error| {
        format!(
            "Could not move {} to {}: {e}",
            file.display(),
            folder.display()
        )
    };
    // A rename replaces the old file in one step. Across drives it fails:
    // then the file is copied next to the old one first, so a failed copy
    // never leaves the old file half overwritten.
    if std::fs::rename(file, &target).is_err() {
        let mut staging = target.clone().into_os_string();
        staging.push(".keepclip-part");
        let staging = PathBuf::from(staging);
        if let Err(e) = std::fs::copy(file, &staging) {
            let _ = std::fs::remove_file(&staging);
            return Err(fail(e).into());
        }
        std::fs::rename(&staging, &target).map_err(fail)?;
        let _ = std::fs::remove_file(file);
    }
    Ok(Moved::Saved(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A work folder and a download folder of their own, removed at the end.
    struct Folders {
        root: PathBuf,
        work: PathBuf,
        downloads: PathBuf,
    }

    impl Folders {
        fn new(test: &str) -> Folders {
            let root =
                std::env::temp_dir().join(format!("keepclip-test-{test}-{}", std::process::id()));
            let (work, downloads) = (root.join("work"), root.join("downloads"));
            std::fs::create_dir_all(&work).unwrap();
            std::fs::create_dir_all(&downloads).unwrap();
            Folders {
                root,
                work,
                downloads,
            }
        }
    }

    impl Drop for Folders {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn a_finished_file_moves_to_the_download_folder() {
        let folders = Folders::new("move");
        let finished = folders.work.join("A [x].mp4");
        std::fs::write(&finished, "new").unwrap();
        let saved = move_into(&finished, &folders.downloads, false).unwrap();
        let target = folders.downloads.join("A [x].mp4");
        assert_eq!(saved, Moved::Saved(target.clone()));
        assert_eq!(std::fs::read_to_string(target).unwrap(), "new");
        assert!(!finished.exists());
    }

    #[test]
    fn a_file_with_the_same_name_stays_unless_replacing_is_allowed() {
        let folders = Folders::new("exists");
        let target = folders.downloads.join("A [x].mp4");
        std::fs::write(&target, "old").unwrap();
        let finished = folders.work.join("A [x].mp4");
        std::fs::write(&finished, "new").unwrap();
        let moved = move_into(&finished, &folders.downloads, false).unwrap();
        assert_eq!(moved, Moved::Exists(target.clone()));
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "old");
        assert!(finished.exists());
    }

    #[test]
    fn an_agreed_replacement_swaps_the_file_once_the_new_one_is_done() {
        let folders = Folders::new("replace");
        std::fs::write(folders.downloads.join("A [x].mp4"), "old").unwrap();
        let finished = folders.work.join("A [x].mp4");
        std::fs::write(&finished, "new").unwrap();
        let Moved::Saved(saved) = move_into(&finished, &folders.downloads, true).unwrap() else {
            panic!("the file should have moved");
        };
        assert_eq!(std::fs::read_to_string(saved).unwrap(), "new");
        assert!(!finished.exists());
        let left: Vec<_> = std::fs::read_dir(&folders.downloads)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(left, ["A [x].mp4"]);
    }
}
