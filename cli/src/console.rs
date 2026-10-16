//! Questions, the progress line and the console window.

use std::io::{self, BufRead, Write};

/// Prints `prompt` and reads one line, without the line break and the
/// spaces around it.
pub fn ask(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line)? == 0 {
        // The input was closed: nothing more will come.
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "no answer: the input is closed",
        ));
    }
    Ok(line.trim().to_string())
}

/// Asks for a number from 1 to `count`. Enter alone picks 1; 0 cancels and
/// returns `None`.
pub fn pick_number(question: &str, count: usize) -> io::Result<Option<usize>> {
    loop {
        let typed = ask(&format!(
            "\n{question} (1-{count}, Enter = 1, 0 = cancel): "
        ))?;
        if typed.is_empty() {
            return Ok(Some(1));
        }
        match typed.parse::<usize>() {
            Ok(0) => return Ok(None),
            Ok(n) if n <= count => return Ok(Some(n)),
            _ => println!("Type a number from 1 to {count}."),
        }
    }
}

pub fn wait_for_enter(message: &str) {
    // Nothing to do if the input is closed: the window closes either way.
    let _ = ask(&format!("\n{message}"));
}

/// "[########------------]", empty when the fraction is unknown.
pub fn bar(fraction: Option<f64>) -> String {
    const WIDTH: usize = 20;
    let filled = fraction.map_or(0, |f| (f.clamp(0.0, 1.0) * WIDTH as f64).round() as usize);
    format!("[{}{}]", "#".repeat(filled), "-".repeat(WIDTH - filled))
}

/// One line that keeps being redrawn, such as a progress bar.
#[derive(Default)]
pub struct StatusLine {
    /// Characters on screen, to blank out what a shorter text leaves.
    shown: usize,
}

impl StatusLine {
    pub fn show(&mut self, text: &str) {
        let width = text.chars().count();
        let blank = self.shown.saturating_sub(width);
        print!("\r{text}{}", " ".repeat(blank));
        let _ = io::stdout().flush();
        self.shown = width;
    }

    /// Leaves the line as it is and moves to the next one.
    pub fn finish(&mut self) {
        if self.shown > 0 {
            println!();
            self.shown = 0;
        }
    }
}

/// Whether keepclip has the console window to itself: started from a
/// shortcut, File Explorer or another app rather than from a terminal. Then
/// the window closes when keepclip ends, so it waits before closing on an
/// error.
#[cfg(windows)]
pub fn is_own_window() -> bool {
    use windows::Win32::System::Console::GetConsoleProcessList;
    let mut processes = [0u32; 4];
    // SAFETY: the buffer outlives the call, which writes at most its length.
    let count = unsafe { GetConsoleProcessList(&mut processes) };
    count == 1
}

#[cfg(not(windows))]
pub fn is_own_window() -> bool {
    false
}
