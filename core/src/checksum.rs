//! Reads the checksum files published next to the tools keepclip downloads.
//!
//! Two layouts are common: one line per file, `<hash>  <name>` (a `*` may come
//! before the name), and the output of PowerShell's `Get-FileHash`, with one
//! `Hash : <HASH>` line for a single file.

/// The SHA-256 of `file_name` in `text`, in lowercase hex. A file with a
/// single hash is taken to be about `file_name`.
pub fn find_sha256(text: &str, file_name: &str) -> Option<String> {
    let named = text.lines().find_map(|line| {
        let mut words = line.split_whitespace();
        let hash = words.next().filter(|w| is_sha256(w))?;
        let name = words.next()?.trim_start_matches('*');
        (name == file_name).then(|| hash.to_ascii_lowercase())
    });
    named.or_else(|| {
        let mut hashes = text
            .split(|c: char| !c.is_ascii_hexdigit())
            .filter(|w| is_sha256(w));
        match (hashes.next(), hashes.next()) {
            (Some(only), None) => Some(only.to_ascii_lowercase()),
            _ => None,
        }
    })
}

fn is_sha256(word: &str) -> bool {
    word.len() == 64 && word.bytes().all(|b| b.is_ascii_hexdigit())
}
