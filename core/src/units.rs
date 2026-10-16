//! Sizes, speeds and times as people read them.

const KB: f64 = 1024.0;
const MB: f64 = KB * 1024.0;
const GB: f64 = MB * 1024.0;

/// "820 bytes", "512 KB", "48.2 MB", "1.24 GB". Multiples of 1024, as File
/// Explorer counts them.
pub fn bytes(n: u64) -> String {
    let n_f = n as f64;
    if n_f < KB {
        format!("{n} bytes")
    } else if n_f < MB {
        format!("{:.0} KB", n_f / KB)
    } else if n_f < GB {
        format!("{:.1} MB", n_f / MB)
    } else {
        format!("{:.2} GB", n_f / GB)
    }
}

/// "2.4 MB/s".
pub fn speed(bytes_per_second: f64) -> String {
    format!("{}/s", bytes(bytes_per_second as u64))
}

/// "0:42", "3:05", "1:02:03".
pub fn duration(seconds: u64) -> String {
    let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
