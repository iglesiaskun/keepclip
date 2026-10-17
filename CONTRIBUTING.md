# Contributing to keepclip

Thanks for helping! This guide explains how changes get into the project.

## Workflow

1. **One branch per change.** Branch off `main` and name it
   `type/short-description`, for example `feat/subtitles`, `fix/clipboard`
   or `docs/readme`.
2. **Keep pull requests small.** One topic per pull request. A rename, a
   translation and a new feature are three pull requests, not one.
3. **Pull requests are squash merged.** All commits in a pull request become a
   single commit on `main`, and its message comes from the pull request title
   and description. Write the title in the imperative mood: "Add subtitles",
   not "Added subtitles".
4. **Delete the branch after the merge.** A squash merge creates a new commit,
   so continuing on the old branch leads to conflicts. Start a new branch from
   `main` instead.
5. **`main` is protected.** Changes land only through pull requests, after CI
   passes.

## Bugs and ideas

Open an issue with the bug report or feature request template. Please search
the open issues first. Security problems are the exception: report them
privately, as [SECURITY.md](SECURITY.md) explains.

A download that fails on one site is often a yt-dlp matter: keepclip updates
yt-dlp every week, and the [yt-dlp issues](https://github.com/yt-dlp/yt-dlp/issues)
may already have it. Report it here when keepclip shows the problem in a way
yt-dlp alone does not.

## Before opening a pull request

If people will notice the change, add a line to [CHANGELOG.md](CHANGELOG.md)
under "Unreleased", in the section that fits (Added, Changed, Fixed, ...).

Tests that need what yt-dlp prints use fixtures in `core/tests/fixtures`.
Real output is welcome, but replace the title, the uploader, the id and every
link with sample values first: links can carry your IP address.

A new dependency needs a reason, written as a comment above it in
`Cargo.toml`, like the others, and in the pull request. Its license must allow
keepclip to stay MIT OR Apache-2.0: no GPL.

Run the same checks CI runs:

```bash
cargo fmt --all
```

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

```bash
cargo test --all
```

## Language

Code, comments, the text keepclip shows, docs, commit messages and pull
request titles are written in English. The one exception is
[README.pt-BR.md](README.pt-BR.md), the Portuguese translation of the README:
a change to one README goes into both. Issues and discussions may be in
English or Portuguese.

## License

By contributing, you agree that your contributions are dual licensed under
the MIT and Apache-2.0 licenses, as described in the [README](README.md#license),
without any additional terms or conditions.
