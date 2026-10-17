//! The questions of the first run: the download folder, and whether to
//! download the programs keepclip needs.

use crate::console::{self, StatusLine};
use keepclip_core::config::Config;
use keepclip_core::units;
use keepclip_system::stop::Stop;
use keepclip_system::tools::{self, Step, Tool};
use keepclip_system::ytdlp::YtDlp;
use keepclip_system::{Result, settings};
use std::path::{Path, PathBuf};

/// Asks for the download folder, creates it and saves it in the settings.
pub fn choose_folder(file: &Path, config: &mut Config) -> Result<PathBuf> {
    let suggestion = settings::suggested_folder(config)?;
    println!("\nWhere should keepclip save the downloads?");
    let folder = loop {
        let typed = console::ask(&format!(
            "Press Enter for {}, or type another folder: ",
            suggestion.display()
        ))?;
        let folder = if typed.is_empty() {
            suggestion.clone()
        } else {
            PathBuf::from(expand_variables(typed.trim_matches('"'), |name| {
                std::env::var(name).ok()
            }))
        };
        if !folder.is_absolute() {
            println!("Type the whole path, such as C:\\Users\\you\\Videos.");
            continue;
        }
        match settings::use_folder(file, config, &folder) {
            Ok(()) => break folder,
            Err(e) => println!("{e}"),
        }
    };
    println!("Saved in {}.", file.display());
    Ok(folder)
}

/// yt-dlp, installed and recent, with ffmpeg and Deno next to it. Asks
/// before downloading what is missing.
pub fn ready_tools() -> Result<YtDlp> {
    let bin = tools::bin_folder()?;
    let missing = tools::missing(&bin);
    if !missing.is_empty() {
        install(&bin, &missing)?;
    }
    if tools::update_due(&bin) {
        println!("Checking for a newer yt-dlp...");
        match tools::update(&bin) {
            Ok(line) if !line.is_empty() => println!("{line}"),
            Ok(_) => {}
            Err(reason) => println!("Could not update yt-dlp; going on with this one. {reason}"),
        }
    }
    Ok(tools::ytdlp(&bin))
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
    for tool in missing {
        let mut line = StatusLine::default();
        let mut shown = None;
        let mut on_step = |step| match step {
            Step::Download { done, total } => {
                // Redraw once per megabyte, not on every read.
                let megabyte = done / (1024 * 1024);
                if shown == Some(megabyte) && Some(done) != total {
                    return;
                }
                shown = Some(megabyte);
                let fraction = total.map(|t| done as f64 / t.max(1) as f64);
                let mut text = format!("{:<7} {}", tool.name, console::bar(fraction));
                text.push_str(&format!(" {}", units::bytes(done)));
                if let Some(total) = total {
                    text.push_str(&format!(" of {}", units::bytes(total)));
                }
                line.show(&text);
            }
            Step::Check | Step::Unpack => line.finish(),
        };
        tools::install(bin, tool, &mut on_step, &Stop::new())?;
        line.finish();
    }
    println!("All set.");
    Ok(())
}

/// Replaces `%NAME%` with the value of the environment variable, as Windows
/// does. Unknown names stay as they are.
fn expand_variables(text: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('%') else { break };
        let name = &after[..end];
        out.push_str(&rest[..start]);
        match lookup(name).filter(|_| !name.is_empty()) {
            Some(value) => {
                out.push_str(&value);
                rest = &after[end + 1..];
            }
            None => {
                // Keep the first % and look for a name from the second one.
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(name: &str) -> Option<String> {
        (name == "USERPROFILE").then(|| "C:\\Users\\you".to_string())
    }

    #[test]
    fn expands_known_variables() {
        assert_eq!(
            expand_variables("%USERPROFILE%\\Videos", lookup),
            "C:\\Users\\you\\Videos"
        );
        assert_eq!(expand_variables("%NOPE%\\x", lookup), "%NOPE%\\x");
        assert_eq!(
            expand_variables("100% %USERPROFILE%", lookup),
            "100% C:\\Users\\you"
        );
        assert_eq!(expand_variables("50%", lookup), "50%");
    }
}
