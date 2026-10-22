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

## Building from source

Requirements (Windows):

- Stable Rust with the `x86_64-pc-windows-msvc` target (installed by `rustup`).
- Visual Studio or Build Tools with the **Desktop development with C++**
  workload (provides the MSVC linker and the Windows SDK).

```bash
cargo test
```

```bash
cargo build --release
```

Then run `target\release\keepclip.exe` (the window) or
`target\release\keepclip-cli.exe` (the console). `--dir <folder>` keeps the
settings file in that folder, which is handy for testing without touching
your real settings. The interface of the window, in `window/ui`, is plain
HTML, CSS and JavaScript: there is no Node, npm or other build step.

The release files come from [`packaging/package.ps1`](packaging/package.ps1),
which the release workflow runs; the comment at its top explains how to run
it locally. It needs [WiX Toolset](https://wixtoolset.org) 5 for the `.msi`
and [cargo-about](https://github.com/EmbarkStudios/cargo-about) for the list
of third-party licenses.

## Project layout

```
core/      library (keepclip-core): pure, testable logic with no Windows
           code: reads what yt-dlp reports, builds the menu, the download
           arguments and the settings file, and follows the progress
system/    library (keepclip-system): what both programs share: runs yt-dlp,
           downloads and updates yt-dlp, ffmpeg and Deno, the settings file,
           the clipboard and the notification
window/    the window (keepclip.exe), on Tauri 2, with its interface in ui/
cli/       the console version (keepclip-cli.exe)
packaging/ the installer (WiX), the portable .zip and the list of
           third-party licenses, made by package.ps1
```

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
