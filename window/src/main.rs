//! keepclip: downloads a video from a copied link, in a small window.
//!
//! The interface is the HTML, CSS and JavaScript in `ui/`, shown by WebView2
//! and embedded in the executable. It calls the commands in `commands.rs`;
//! the slow work (yt-dlp, the downloads) runs in other threads and reports
//! back with events.

#![windows_subsystem = "windows"]

mod commands;
mod shell;
mod thumbs;
mod view;

use keepclip_system::args::{self, Command};
use keepclip_system::{settings, tools};
use std::process::ExitCode;
use tauri::{PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// The window's size, in pixels at 100% scaling. It does not change.
const WIDTH: f64 = 440.0;
const HEIGHT: f64 = 560.0;

fn main() -> ExitCode {
    let args = match args::parse(std::env::args().skip(1)) {
        Ok(Command::Run(args)) => args,
        Ok(Command::Help) => {
            shell::message(&args::help("keepclip"), false);
            return ExitCode::SUCCESS;
        }
        Ok(Command::Version) => {
            shell::message(&format!("keepclip {}", env!("CARGO_PKG_VERSION")), false);
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            shell::message(
                &format!("keepclip: {e}\n\n{}", args::help("keepclip")),
                true,
            );
            return ExitCode::FAILURE;
        }
    };
    let state = settings::file(args.dir.as_deref()).and_then(|file| {
        let config = settings::load(&file)?;
        let bin = tools::bin_folder()?;
        Ok(commands::State::new(args, file, config, bin))
    });
    let state = match state {
        Ok(state) => state,
        Err(e) => {
            shell::message(&e.to_string(), true);
            return ExitCode::FAILURE;
        }
    };
    #[cfg(windows)]
    if tauri::webview_version().is_err() {
        shell::message(shell::NO_WEBVIEW, true);
        return ExitCode::FAILURE;
    }
    // The web view's own files, next to the downloaded programs.
    let data_dir = state.bin.with_file_name("window");

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(commands::handler())
        .setup(move |app| {
            let window = WebviewWindowBuilder::new(
                app,
                commands::WINDOW,
                WebviewUrl::App("index.html".into()),
            )
            .title("keepclip")
            .inner_size(WIDTH, HEIGHT)
            .resizable(false)
            .maximizable(false)
            // The interface draws the title bar, with only – and ✕:
            // Windows would add a disabled maximize button.
            .decorations(false)
            .shadow(true)
            .data_directory(data_dir.clone())
            // Shown once the interface is drawn, so it never flashes
            // white.
            .visible(false)
            .build()?;
            center_where_the_mouse_is(&window);
            Ok(())
        })
        .run(tauri::generate_context!());

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            shell::message(&format!("keepclip could not open its window:\n\n{e}"), true);
            ExitCode::FAILURE
        }
    }
}

/// Puts the window in the middle of the screen the mouse is on: keepclip
/// opens from a mouse action or a shortcut, where the user is looking.
fn center_where_the_mouse_is(window: &WebviewWindow) {
    let Ok(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(Some(monitor)) = window.monitor_from_point(cursor.x, cursor.y) else {
        return;
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let (width, height) = ((WIDTH * scale) as i32, (HEIGHT * scale) as i32);
    let x = area.position.x + (area.size.width as i32 - width) / 2;
    let y = area.position.y + (area.size.height as i32 - height) / 2;
    let _ = window.set_position(PhysicalPosition::new(x, y));
    fit(window);
}

/// Gives the window its size inside. Without a title bar of Windows', the
/// size given when it is made still counts one; the hidden frame is only
/// known once the window exists, so this runs again before it is shown.
pub fn fit(window: &WebviewWindow) {
    let (Ok(inner), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    let wanted = tauri::LogicalSize::new(WIDTH, HEIGHT).to_physical::<u32>(scale);
    if inner != wanted {
        let _ = window.set_size(wanted);
    }
}
