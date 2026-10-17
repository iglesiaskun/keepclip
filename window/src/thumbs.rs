//! The thumbnails of the videos, fetched after the choices are on screen,
//! so they never hold them up.
//!
//! keepclip fetches each picture itself and hands it to the interface as a
//! `data:` address: the window loads nothing from the web, and its content
//! security policy says so.

use keepclip_core::link;
use keepclip_system::fetch;
use serde::Serialize;
use std::thread;
use tauri::{Emitter, WebviewWindow};

/// Width to ask for: the box is 128 pixels wide, and twice that on a screen
/// scaled to 200%.
pub const WIDTH: u64 = 256;

/// Bigger pictures are not worth the wait.
const MAX_BYTES: u64 = 4 * 1024 * 1024;

/// Addresses tried per video, at most.
const TRIES: usize = 3;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Thumb {
    /// The reading the video belongs to.
    read: u64,
    video: usize,
    /// The picture as a `data:` address, or `None` when none came.
    src: Option<String>,
}

/// Fetches a picture for each video, in a thread, and sends each one as a
/// "thumb" event. `videos` holds, per video, the addresses to try in order.
pub fn fetch(window: WebviewWindow, read: u64, videos: Vec<Vec<String>>) {
    thread::spawn(move || {
        for (video, urls) in videos.into_iter().enumerate() {
            let src = urls
                .iter()
                .filter(|url| link::is_web_link(url))
                .take(TRIES)
                .find_map(|url| data_address(url));
            if window.emit("thumb", Thumb { read, video, src }).is_err() {
                // The window is gone.
                break;
            }
        }
    });
}

fn data_address(url: &str) -> Option<String> {
    let bytes = fetch::bytes(url, MAX_BYTES).ok()?;
    let kind = image_type(&bytes)?;
    Some(format!("data:{kind};base64,{}", base64(&bytes)))
}

/// The type of a picture from its first bytes. The server's word is not
/// enough: an error page can come back as "image/jpeg".
fn image_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(b"GIF8") {
        Some("image/gif")
    } else if bytes.len() > 12 && &bytes[4..8] == b"ftyp" && &bytes[8..12] == b"avif" {
        Some("image/avif")
    } else {
        None
    }
}

/// Standard Base64, with padding.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for group in bytes.chunks(3) {
        let n = group
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= group.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_standard_examples() {
        // RFC 4648, section 10.
        let cases = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (text, encoded) in cases {
            assert_eq!(base64(text.as_bytes()), encoded);
        }
        assert_eq!(base64(&[0xFF, 0xEF]), "/+8=");
    }

    #[test]
    fn pictures_are_known_by_their_first_bytes() {
        assert_eq!(image_type(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(image_type(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
        assert_eq!(image_type(b"<!doctype html>"), None);
    }
}
