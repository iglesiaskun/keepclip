//! Reading the checksum files published with the tools.

use keepclip_core::checksum::find_sha256;

/// One line per file, as `sha256sum` writes it.
const LIST: &str = include_str!("fixtures/sums-lines.txt");
/// A single hash, as PowerShell's `Get-FileHash` writes it.
const SINGLE: &str = include_str!("fixtures/sums-get-filehash.txt");

#[test]
fn finds_the_named_file_in_a_list() {
    assert_eq!(
        find_sha256(LIST, "tool.exe").as_deref(),
        Some("66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a")
    );
    // Not "tool_arm64.exe" nor "tool", which only start or end alike.
    assert_eq!(
        find_sha256(LIST, "tool").as_deref(),
        Some("1fa6733c37ea6fb51c99ad8fe785e7b7e5f3246c9b980230329d4fb72ed8d4d6")
    );
}

#[test]
fn accepts_the_binary_mode_star() {
    assert_eq!(
        find_sha256(LIST, "tool-win64.zip").as_deref(),
        Some("c32269c715f932b45487702cfc238bca54945a63957efa3263f025a273000e99")
    );
}

#[test]
fn a_list_without_the_file_gives_nothing() {
    assert_eq!(find_sha256(LIST, "other.exe"), None);
}

#[test]
fn a_single_hash_belongs_to_the_file_and_comes_in_lowercase() {
    assert_eq!(
        find_sha256(SINGLE, "tool-x86_64-pc-windows-msvc.zip").as_deref(),
        Some("a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238")
    );
    let with_crlf = SINGLE.replace('\n', "\r\n");
    assert!(find_sha256(&with_crlf, "any.zip").is_some());
}

#[test]
fn text_without_a_hash_gives_nothing() {
    assert_eq!(find_sha256("Not Found", "tool.exe"), None);
    assert_eq!(find_sha256("", "tool.exe"), None);
}
