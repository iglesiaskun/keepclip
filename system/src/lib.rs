//! What the window (`keepclip.exe`) and the console (`keepclip-cli.exe`)
//! share: the command line, running yt-dlp, downloading the tools, the
//! settings file, moving the finished file, the clipboard and the
//! notification. The logic that needs no operating system is in
//! `keepclip-core`; asking the user is left to each program.
//!
//! Windows-only code stays behind `cfg(windows)`, with a fallback for other
//! systems.

pub mod args;
pub mod clipboard;
pub mod fetch;
pub mod files;
mod job;
pub mod notify;
pub mod settings;
pub mod stop;
pub mod tools;
pub mod ytdlp;

/// An error with a message for the user. `Send` and `Sync`, so the window
/// can carry it from the thread that did the work.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Starts a program without a console window of its own. The window has
/// no console, so without this each yt-dlp would open one.
pub(crate) fn no_console(command: &mut std::process::Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = command;
}
