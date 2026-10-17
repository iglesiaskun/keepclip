//! Runs yt-dlp: once to read what a link offers, once to download.

use crate::Result;
use crate::job::Job;
use crate::stop::{Stop, Stopped};
use keepclip_core::download::{self, Request, Tools};
use keepclip_core::info::VideoInfo;
use keepclip_core::progress::{self, Event};
use std::ffi::OsString;
use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, mpsc};
use std::thread;

/// Lines of yt-dlp's messages kept to explain a failure.
const KEPT_MESSAGES: usize = 20;

/// Added to a failure when the site seems to want a login.
pub const LOGIN_HINT: &str =
    "The site may want you to be logged in to see this video. keepclip cannot log in to sites yet.";

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
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        crate::no_console(&mut command);
        command
    }

    /// Starts yt-dlp in a job that `stop` ends.
    fn start(&self, args: Vec<OsString>, stop: &Stop) -> Result<Running> {
        stop.check()?;
        let child = self
            .command(args)
            .spawn()
            .map_err(|e| format!("Could not run yt-dlp: {e}"))?;
        let job = Job::for_child(&child).map(Arc::new);
        if let Some(job) = &job {
            stop.watch(Arc::clone(job));
        }
        Ok(Running {
            child,
            _job: job,
            stop: stop.clone(),
        })
    }

    /// Runs yt-dlp to its end and returns what it printed.
    fn output(&self, args: Vec<OsString>, stop: &Stop) -> Result<Output> {
        let mut running = self.start(args, stop)?;
        let child = &mut running.child;
        let stdout = child.stdout.take().map(read_all);
        let stderr = child.stderr.take().map(read_all);
        let status = child.wait()?;
        let collect = |reader: Option<thread::JoinHandle<Vec<u8>>>| {
            reader
                .and_then(|reader| reader.join().ok())
                .unwrap_or_default()
        };
        let output = Output {
            status,
            stdout: collect(stdout),
            stderr: collect(stderr),
        };
        stop.check()?;
        Ok(output)
    }

    /// What the link offers (`yt-dlp -J`), read, and the JSON as yt-dlp
    /// printed it, for [`YtDlp::final_name`].
    pub fn info(&self, url: &str, stop: &Stop) -> Result<(VideoInfo, String)> {
        let output = self.output(download::info_args(url, &self.tools), stop)?;
        if !output.status.success() {
            let messages = String::from_utf8_lossy(&output.stderr);
            return Err(Failed::from_messages(messages.lines()).into());
        }
        let json = String::from_utf8_lossy(&output.stdout).into_owned();
        Ok((VideoInfo::parse(&json)?, json))
    }

    /// The name the downloaded file will have, asked from yt-dlp without
    /// downloading: it reads `json`, the output of [`YtDlp::info`], from a
    /// file in the work folder. `None` when yt-dlp could not say.
    pub fn final_name(&self, json: &str, request: &Request<'_>) -> Option<String> {
        let file = request
            .work_folder
            .join(format!("{}.info.json", std::process::id()));
        std::fs::write(&file, json).ok()?;
        let output = self.output(
            download::name_args(request, &file, &self.tools),
            &Stop::new(),
        );
        let _ = std::fs::remove_file(&file);
        let output = output.ok().filter(|o| o.status.success())?;
        download::final_file_name(&String::from_utf8_lossy(&output.stdout), request.choice)
    }

    /// Downloads, calling `on_event` for each progress line. Ends with
    /// [`Stopped`] when `stop` stops it.
    pub fn download(
        &self,
        request: &Request<'_>,
        stop: &Stop,
        mut on_event: impl FnMut(Event),
    ) -> Result<()> {
        let mut running = self.start(download::download_args(request, &self.tools), stop)?;
        let child = &mut running.child;

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
        stop.check()?;
        if status.success() {
            Ok(())
        } else {
            Err(Failed::from_messages(messages.iter().map(String::as_str)).into())
        }
    }

    /// `yt-dlp --version`: "2026.08.19".
    pub fn version(&self) -> Option<String> {
        let output = self
            .output(vec![OsString::from("--version")], &Stop::new())
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().next()?.trim();
        (output.status.success() && !line.is_empty()).then(|| line.to_string())
    }
}

/// A yt-dlp that is running, in its job when Windows gave one.
struct Running {
    child: Child,
    _job: Option<Arc<Job>>,
    stop: Stop,
}

impl Drop for Running {
    fn drop(&mut self) {
        self.stop.unwatch();
    }
}

/// Reads a whole stream in a thread of its own, so a full pipe never holds
/// up the other one.
fn read_all(mut stream: impl Read + Send + 'static) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stream.read_to_end(&mut bytes);
        bytes
    })
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

/// yt-dlp ended with an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failed {
    /// Its error lines without the "[site] id: " in front, or its last
    /// messages when it gave no error line.
    pub said: String,
    /// `said` holds error lines, not just the last messages.
    pub errors: bool,
    /// The words look like the site wants a login: see [`LOGIN_HINT`].
    pub login: bool,
}

impl Failed {
    fn from_messages<'a>(messages: impl Iterator<Item = &'a str>) -> Failed {
        let messages: Vec<&str> = messages.collect();
        let errors: Vec<&str> = messages
            .iter()
            .filter_map(|line| line.strip_prefix("ERROR: "))
            .map(without_source)
            .collect();
        let (said, has_errors) = if errors.is_empty() {
            let last: Vec<&str> = messages.iter().rev().take(5).rev().copied().collect();
            (last.join("\n"), false)
        } else {
            (errors.join("\n"), true)
        };
        let lower = said.to_lowercase();
        let login = ["log in", "login", "sign in", "cookies", "private"]
            .iter()
            .any(|word| lower.contains(word));
        Failed {
            said,
            errors: has_errors,
            login,
        }
    }
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.errors {
            write!(f, "yt-dlp could not do it:\n{}", self.said)?;
        } else {
            write!(f, "yt-dlp stopped with an error.\n{}", self.said)?;
        }
        if self.login {
            write!(f, "\n\n{LOGIN_HINT}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Failed {}

/// "[site] video-id: message" becomes "message".
fn without_source(error: &str) -> &str {
    error
        .strip_prefix('[')
        .and_then(|rest| rest.split_once("] "))
        .and_then(|(_, rest)| rest.split_once(": "))
        .map_or(error, |(_, message)| message)
}

/// Whether `error` is the end of work stopped on purpose.
pub fn was_stopped(error: &crate::Error) -> bool {
    error.is::<Stopped>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn explain<'a>(messages: impl Iterator<Item = &'a str>) -> String {
        Failed::from_messages(messages).to_string()
    }

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
        let failed = Failed::from_messages(["ERROR: [site] 1: Log in for access"].into_iter());
        assert!(failed.login);
        assert_eq!(failed.said, "Log in for access");
        assert!(
            failed
                .to_string()
                .ends_with("keepclip cannot log in to sites yet.")
        );
    }

    #[test]
    fn without_an_error_line_the_last_messages_explain() {
        let text = explain(["one", "two"].into_iter());
        assert_eq!(text, "yt-dlp stopped with an error.\none\ntwo");
    }
}
