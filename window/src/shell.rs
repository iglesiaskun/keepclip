//! What keepclip asks Windows to do: show a message, open a file or a
//! folder, and show a file in File Explorer.

use std::path::Path;

/// Shown when WebView2, which draws the window, is missing.
#[cfg(windows)]
pub const NO_WEBVIEW: &str = "keepclip needs the WebView2 Runtime, which comes with Windows 11 \
    and with the updates of Windows 10. Install it from \
    https://developer.microsoft.com/microsoft-edge/webview2/ and open keepclip again.\n\n\
    keepclip-cli.exe, the console version, works without it.";

/// A message box, for what happens before the window exists: the program
/// has no console to write to.
#[cfg(windows)]
pub fn message(text: &str, error: bool) {
    use windows::Win32::UI::WindowsAndMessaging::{
        MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MessageBoxW,
    };
    use windows::core::{HSTRING, w};
    let icon = if error {
        MB_ICONERROR
    } else {
        MB_ICONINFORMATION
    };
    // SAFETY: both strings live until the call returns.
    unsafe {
        MessageBoxW(None, &HSTRING::from(text), w!("keepclip"), MB_OK | icon);
    }
}

#[cfg(not(windows))]
pub fn message(text: &str, _error: bool) {
    eprintln!("{text}");
}

/// Opens a file with its default program, or a folder in File Explorer.
#[cfg(windows)]
pub fn open(path: &Path) -> Result<(), String> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::{HSTRING, PCWSTR, w};
    let _com = Com::init();
    // SAFETY: the strings live until the call returns.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            &HSTRING::from(path.as_os_str()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // Values above 32 mean it worked.
    if result.0 as usize > 32 {
        Ok(())
    } else {
        Err(format!("Windows could not open {}.", path.display()))
    }
}

/// Opens the file's folder in File Explorer, with the file selected.
#[cfg(windows)]
pub fn show_in_folder(file: &Path) -> Result<(), String> {
    use windows::Win32::UI::Shell::{ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems};
    use windows::core::HSTRING;
    let _com = Com::init();
    // SAFETY: the path lives until the call returns; the item it gives is
    // freed below, once.
    let item = unsafe { ILCreateFromPathW(&HSTRING::from(file.as_os_str())) };
    if item.is_null() {
        return Err(format!("Windows could not find {}.", file.display()));
    }
    // With no items listed, the first argument is the item to select, in
    // its folder.
    // SAFETY: `item` is valid until freed.
    let result = unsafe { SHOpenFolderAndSelectItems(item, None, 0) };
    // SAFETY: made by ILCreateFromPathW above and not used after this.
    unsafe { ILFree(Some(item)) };
    result.map_err(|e| format!("Windows could not show {}: {e}", file.display()))
}

/// COM for the calls above, which need it on the thread that makes them.
#[cfg(windows)]
struct Com {
    started: bool,
}

#[cfg(windows)]
impl Com {
    fn init() -> Com {
        use windows::Win32::System::Com::{
            COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx,
        };
        // SAFETY: balanced by CoUninitialize on drop when it succeeds; a
        // thread that has COM already just keeps it.
        let result =
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
        Com {
            started: result.is_ok(),
        }
    }
}

#[cfg(windows)]
impl Drop for Com {
    fn drop(&mut self) {
        if self.started {
            // SAFETY: matches the successful CoInitializeEx above.
            unsafe { windows::Win32::System::Com::CoUninitialize() };
        }
    }
}

#[cfg(not(windows))]
pub fn open(path: &Path) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(not(windows))]
pub fn show_in_folder(file: &Path) -> Result<(), String> {
    open(file.parent().unwrap_or(file))
}
