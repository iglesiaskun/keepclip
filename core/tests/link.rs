//! Finding the link in what the user copied.

use keepclip_core::link::{find, is_web_link};

#[test]
fn a_bare_link_is_itself() {
    assert_eq!(
        find("https://video.example/watch?v=abc123"),
        Some("https://video.example/watch?v=abc123")
    );
}

#[test]
fn finds_the_link_inside_shared_text() {
    let shared = "Look at this! https://video.example/v/42?s=share#t=10 (so funny)";
    assert_eq!(
        find(shared),
        Some("https://video.example/v/42?s=share#t=10")
    );
    assert_eq!(
        find("\r\n  https://video.example/v/1\r\n"),
        Some("https://video.example/v/1")
    );
}

#[test]
fn drops_punctuation_after_the_link() {
    assert_eq!(
        find("Watch https://video.example/v/1."),
        Some("https://video.example/v/1")
    );
    assert_eq!(
        find("(https://video.example/v/1)"),
        Some("https://video.example/v/1")
    );
    assert_eq!(
        find("\"https://video.example/v/1\","),
        Some("https://video.example/v/1")
    );
}

#[test]
fn takes_the_first_of_several_links() {
    assert_eq!(
        find("http://a.example/1 https://b.example/2"),
        Some("http://a.example/1")
    );
}

#[test]
fn ignores_text_without_a_web_link() {
    assert_eq!(find("just some words"), None);
    assert_eq!(find("ftp://files.example/v.mp4"), None);
    assert_eq!(find("https://"), None);
    assert_eq!(find(""), None);
}

#[test]
fn a_web_link_needs_a_host() {
    assert!(is_web_link("https://video.example"));
    assert!(is_web_link("HTTP://VIDEO.EXAMPLE/X"));
    assert!(!is_web_link("https:///path"));
    assert!(!is_web_link("https://?q=1"));
    assert!(!is_web_link("video.example/x"));
    // Options for yt-dlp are not links.
    assert!(!is_web_link("--exec"));
}
