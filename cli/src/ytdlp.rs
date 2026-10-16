//! Runs yt-dlp: once to read what a link offers, once to download.

use crate::Result;
use keepclip_core::download::{self, Request, Tools};
use keepclip_core::info::VideoInfo;
use keepclip_core::progress::{self, Event};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

/// Lines of yt-dlp's messages kept to explain a failure.
const KEPT_MESSAGES: usize = 20;

pub struct YtDlp {
    program: PathBuf,
    tools: Tools,
}

impl YtDlp {
    pub fn new(program: PathBuf, tools: Tools) -> YtDlp {
        YtDlp { program, tools }
    }

    fn command(&self, args: Vec<OsString>) -> Command {
        let mut command = Command::new(&self.program);
        command
            .args(args)
            // yt-dlp is a Python program: UTF-8 everywhere, whatever the
            // language of Windows.
            .env("PYTHONUTF8", "1")
            .env("PYTHONIOENCODING", "utf-8")
            .stdin(Stdio::null());
        command
    }

    /// What the link offers (`yt-dlp -J`).
    pub fn info(&self, url: &str) -> Result<VideoInfo> {
        let output = self
            .command(download::info_args(url, &self.tools))
            .output()
            .map_err(|e| format!("Could not run yt-dlp: {e}"))?;
        if !output.status.success() {
            let messages = String::from_utf8_lossy(&output.stderr);
            return Err(explain(messages.lines()).into());
        }
        let json = String::from_utf8_lossy(&output.stdout);
        Ok(VideoInfo::parse(&json)?)
    }

    /// Downloads, calling `on_event` for each progress line.
    pub fn download(&self, request: &Request<'_>, mut on_event: impl FnMut(Event)) -> Result<()> {
        let mut child = self
            .command(download::download_args(request, &self.tools))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Could not run yt-dlp: {e}"))?;

        // yt-dlp writes progress to one stream and messages to the other;
        // which goes where depends on its options. Both are read, line by
        // line, and sent here in the order they come.
        let (lines, received) = mpsc::channel::<String>();
        let readers = [
            child.stdout.take().map(|out| forward(out, lines.clone())),
            child.stderr.take().map(|err| forward(err, lines.clone())),
        ];
        drop(lines);

        let mut messages: Vec<String> = Vec::new();
        for line in received {
            match progress::parse(&line) {
                Some(event) => on_event(event),
                None if !line.trim().is_empty() => {
                    if messages.len() == KEPT_MESSAGES {
                        messages.remove(0);
                    }
                    messages.push(line);
                }
                None => {}
            }
        }
        for reader in readers.into_iter().flatten() {
            let _ = reader.join();
        }
        let status = child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(explain(messages.iter().map(String::as_str)).into())
        }
    }
}

fn forward(
    stream: impl Read + Send + 'static,
    lines: mpsc::Sender<String>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        let mut line = Vec::new();
        // Bytes, not `lines()`: one line that is not valid UTF-8 must not end
        // the reading.
        while matches!(reader.read_until(b'\n', &mut line), Ok(n) if n > 0) {
            let text = String::from_utf8_lossy(&line).trim_end().to_string();
            if lines.send(text).is_err() {
                break;
            }
            line.clear();
        }
    })
}

/// The error yt-dlp gave, as a message for the user.
fn explain<'a>(messages: impl Iterator<Item = &'a str>) -> String {
    let messages: Vec<&str> = messages.collect();
    let errors: Vec<&str> = messages
        .iter()
        .filter_map(|line| line.strip_prefix("ERROR: "))
        .map(without_source)
        .collect();
    let mut text = if errors.is_empty() {
        let last = messages
            .iter()
            .rev()
            .take(5)
            .rev()
            .copied()
            .collect::<Vec<_>>();
        format!("yt-dlp stopped with an error.\n{}", last.join("\n"))
    } else {
        format!("yt-dlp could not do it:\n{}", errors.join("\n"))
    };
    let lower = text.to_lowercase();
    if ["log in", "login", "sign in", "cookies", "private"]
        .iter()
        .any(|word| lower.contains(word))
    {
        text.push_str(
            "\n\nThe site may want you to be logged in to see this video. keepclip cannot \
             log in to sites yet.",
        );
    }
    text
}

/// "[site] video-id: message" becomes "message".
fn without_source(error: &str) -> &str {
    error
        .strip_prefix('[')
        .and_then(|rest| rest.split_once("] "))
        .and_then(|(_, rest)| rest.split_once(": "))
        .map_or(error, |(_, message)| message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_lose_the_site_and_id() {
        let text = explain(
            [
                "WARNING: something minor",
                "ERROR: [site] abc123: This video is unavailable",
            ]
            .into_iter(),
        );
        assert_eq!(text, "yt-dlp could not do it:\nThis video is unavailable");
        assert_eq!(
            without_source("Unsupported URL: https://example.com"),
            "Unsupported URL: https://example.com"
        );
    }

    #[test]
    fn a_login_wall_gets_a_hint() {
        let text = explain(["ERROR: [site] 1: Log in for access"].into_iter());
        assert!(text.ends_with("keepclip cannot log in to sites yet."));
    }

    #[test]
    fn without_an_error_line_the_last_messages_explain() {
        let text = explain(["one", "two"].into_iter());
        assert_eq!(text, "yt-dlp stopped with an error.\none\ntwo");
    }
}
