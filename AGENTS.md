# AGENTS.md

Guidance for AI coding agents working on keepclip. Everything in
[CONTRIBUTING.md](CONTRIBUTING.md) applies to agents too: the branch and pull
request workflow, the checks CI runs, the changelog, fixtures, new
dependencies and the language of code and commits. What keepclip does and
how to build it are in [README.md](README.md). This file adds what an agent
needs on top.

## Build and check

Stable Rust, no older than `rust-version` in the root `Cargo.toml`; on
Windows, the `x86_64-pc-windows-msvc` target. Before each commit, run the
three checks in CONTRIBUTING.md ("Before opening a pull request"): they are
the ones CI runs, and CI fails on any clippy warning.

## Where code goes

- `core/` (`keepclip-core`): pure logic with no Windows code and no I/O:
  reading `yt-dlp -J`, the menu, the yt-dlp arguments, the progress lines,
  checksums, links and the `config.toml` format. Logic that can live without
  the operating system goes here, with its tests in `core/tests/`.
- `system/` (`keepclip-system`): what both programs share and what needs
  the operating system: the command line, running yt-dlp, downloading the
  tools, the settings file, moving the finished file, the clipboard and the
  notification. Asking the user is left to each program. Windows-only code
  stays behind `cfg(windows)`, with a fallback for other systems.
- `window/` (`keepclip.exe`): the window, on Tauri 2. Its interface in
  `window/ui/` is plain HTML, CSS and JavaScript, loaded as they are: no
  Node, no npm, no build step. `window/build.rs` draws the app icon.
- `cli/` (`keepclip-cli.exe`): the console version: its questions and its
  progress line.
- `packaging/`: the release files. `package.ps1` makes the `.msi` from
  `keepclip.wxs` (WiX 5), the `.zip` and the third-party notices
  (cargo-about); `.github/workflows/release.yml` runs it for a tag. The
  `UpgradeCode` and the component GUIDs in `keepclip.wxs` never change.

## Invariants

- yt-dlp runs as a separate program through `std::process::Command`. Do not
  add the `yt-dlp` crate or any GPL dependency: keepclip is MIT OR
  Apache-2.0.
- ffmpeg, yt-dlp and Deno are never shipped with keepclip. They are
  downloaded on the first run, after the user agrees, and each file is
  checked against the SHA-256 published with it before it is used.
- The link always goes after `--` on the yt-dlp command line, so it can
  never be read as an option.
- yt-dlp runs with `--ignore-config`: keepclip reads its output and must
  know its shape.
- yt-dlp runs in a job object (`system/src/job.rs`), so Cancel, or closing
  keepclip, ends it together with the programs it started.
- The window loads nothing from the web: its content security policy in
  `window/tauri.conf.json` allows only its own files and `data:` pictures.
  keepclip fetches the thumbnails itself and hands them over as `data:`
  addresses. Text that comes from a site goes through `esc()` in
  `window/ui/app.js` before it becomes HTML.
- The window's capability (`window/capabilities/default.json`) lists only
  the Tauri permissions the interface calls. A new call from `app.js` adds
  its own permission there, never a whole default set.
- The progress templates and `progress::parse` change together, with the
  fixture in `core/tests/fixtures/progress-lines.txt`.
- A change that would make an older `config.toml` mean something else raises
  `FORMAT_VERSION` (`core/src/config.rs`) and converts older files in
  `config::parse`, with a test.
- Generic names only: code, tests, examples and docs name no brand or product
  of another company, and no video site. Describe sites as "the sites yt-dlp
  supports". The exceptions are what building and running need: Windows,
  WebView2, GitHub, the build tools, the programs keepclip downloads (yt-dlp,
  ffmpeg, Deno) and the Lucide credit.
- Icons come only from [Lucide](https://lucide.dev), unmodified, in
  `window/ui/icons/`, next to `LICENSE-LUCIDE`.

## Editing

- Edit manifests such as `Cargo.toml` with targeted changes. Rewriting the
  whole file loses its comments and fields such as `license`.
- Comments and docs say what the code does and why. They do not tell how it
  came to be: that belongs in commit messages and pull requests.
- `README.pt-BR.md` is the Portuguese translation of `README.md` and the one
  file not in English. A change to one README goes into both.

## Left to people

Pushes, tags and releases are done by a person: commit on your branch and stop
there.
