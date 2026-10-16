//! Finds the link to download in what the user copied.

/// The first web link in `text`. Copied text often brings more than the
/// link: a title before it, a period or a closing parenthesis after it.
pub fn find(text: &str) -> Option<&str> {
    text.split_whitespace().find_map(|word| {
        let start = word.to_ascii_lowercase().find("http")?;
        let link = word[start..].trim_end_matches(|c: char| {
            matches!(
                c,
                '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '>' | '"' | '\''
            )
        });
        is_web_link(link).then_some(link)
    })
}

/// `http://` or `https://` followed by a host.
pub fn is_web_link(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"));
    rest.is_some_and(|rest| {
        let host = rest.split(['/', '?', '#']).next().unwrap_or("");
        !host.is_empty() && !host.starts_with(['.', ':', '@'])
    })
}
