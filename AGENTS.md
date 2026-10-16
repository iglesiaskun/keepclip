# AGENTS.md

Rules for AI agents working on keepclip. The workflow for every contributor
is in [CONTRIBUTING.md](CONTRIBUTING.md); what keepclip does and how to build
it are in [README.md](README.md).

## Build and check

Stable Rust, no older than `rust-version` in the root `Cargo.toml`; on
Windows, the `x86_64-pc-windows-msvc` target. Before each commit, run what CI
runs:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Where code goes

- `core/` (`keepclip-core`): pure logic with no Windows code and no I/O:
  reading `yt-dlp -J`, the menu, the yt-dlp arguments, the progress lines,
  checksums, links and the `config.toml` format. Logic that can live without
  the operating system goes here, with its tests in `core/tests/`.
- `cli/` (`keepclip.exe`): running yt-dlp, downloading the tools, the
  console, the clipboard and the notification. Windows-only code stays behind
  `cfg(windows)`, with a fallback for other systems.

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
- The progress templates and `progress::parse` change together, with the
  fixture in `core/tests/fixtures/progress-lines.txt`.
- Fixtures from real yt-dlp output have the title, the uploader, the id and
  every link replaced with sample values: links can carry an IP address.
- A change that would make an older `config.toml` mean something else raises
  `FORMAT_VERSION` (`core/src/config.rs`) and converts older files in
  `config::parse`, with a test.
- Generic names only: code, tests, examples and docs name no brand or product
  of another company, and no video site. Describe sites as "the sites yt-dlp
  supports". The exceptions are what building and running need: Windows,
  GitHub, the build tools and the programs keepclip downloads (yt-dlp,
  ffmpeg, Deno).

## Conventions

- English in code, comments, commit messages and docs. `README.pt-BR.md` is
  the one Portuguese file.
- One branch per change (`type/short-description`), small pull requests,
  squash merged, with the title in the imperative ("Add subtitles").
- Edit manifests such as `Cargo.toml` with targeted changes. Rewriting the
  whole file loses its comments and fields such as `license`.
- A new dependency needs a reason, written as a comment above it in
  `Cargo.toml`, like the others, and in the pull request.
- A change people will notice updates `CHANGELOG.md` under "Unreleased", and
  both `README.md` and `README.pt-BR.md`.

## Left to people

Pushes, tags and releases are done by a person: commit on your branch and stop
there.
