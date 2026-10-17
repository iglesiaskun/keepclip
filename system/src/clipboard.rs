//! Reads the text in the clipboard.

/// The text in the clipboard, or `None` when it holds no text or cannot be
/// opened.
#[cfg(windows)]
pub fn text() -> Option<String> {
    use std::time::Duration;
    use windows::Win32::Foundation::HGLOBAL;
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    use windows::Win32::System::Ole::CF_UNICODETEXT;

    // Another app may hold the clipboard for a moment while it writes to it.
    // SAFETY: no window owns the clipboard while keepclip reads it.
    let opened = (0..10).any(|attempt| {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(20));
        }
        unsafe { OpenClipboard(None) }.is_ok()
    });
    if !opened {
        return None;
    }
    // SAFETY: the clipboard is open; the handle it gives stays valid until it
    // is closed, and the memory is read only between lock and unlock, within
    // the size the system reports.
    let text = unsafe {
        GetClipboardData(u32::from(CF_UNICODETEXT.0))
            .ok()
            .and_then(|handle| {
                let memory = HGLOBAL(handle.0);
                let units = GlobalSize(memory) / 2;
                let start = GlobalLock(memory) as *const u16;
                if start.is_null() {
                    return None;
                }
                let all = std::slice::from_raw_parts(start, units);
                let length = all.iter().position(|&unit| unit == 0).unwrap_or(units);
                let text = String::from_utf16_lossy(&all[..length]);
                let _ = GlobalUnlock(memory);
                Some(text)
            })
    };
    // SAFETY: opened above.
    let _ = unsafe { CloseClipboard() };
    text
}

#[cfg(not(windows))]
pub fn text() -> Option<String> {
    None
}
