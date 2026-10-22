# Changelog

All notable changes to keepclip are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before 1.0, a
minor version (0.2.0, 0.3.0) may change the settings file format; when that
happens, keepclip updates older files by itself and this file says what
changed.

## [Unreleased]

## [0.1.0] - 2026-10-22

First public release. keepclip saves a video from a link you copied, on
Windows 10 and 11: open it from the Start menu, click Read link, pick a
quality by its size, and the file lands in your download folder. yt-dlp does
the work.

### Added

- **A short list of choices.** One line per resolution, with the video that
  plays almost everywhere (H.264 and AAC in MP4) where the site has it, and
  Audio only, the video's own sound as a 192 kbps MP3. Each line shows its
  size: exact, estimated (~) by the site or from the bitrate and the length,
  or unknown.
- **A small window** that opens with the link you copied already in its
  field, shows the video with its thumbnail, title, author and length, and
  downloads a choice with a click or its number key. It follows the light or
  dark mode of Windows.
- **Progress you can follow**: each part of the download, also on the
  window's title and its taskbar button. Cancel keeps the parts, so picking
  the same choice again picks up where it stopped. Once the file is saved, the
  window offers Open folder, Play and Download another, and closes by itself
  after 5 seconds, unless the mouse is over it; a Windows notification says
  so when the window is not in front.
- **Your files stay safe**: a file with the same name, such as the same video
  at another resolution, is replaced only if you say so, and only once the new
  one is complete. Each file is made in a temporary folder and moved to yours
  when it is ready.
- **Links with several videos** show a list, up to 20, to pick one from.
- **Errors you can act on**: the reason yt-dlp gave, ready to copy, with Try
  again, and a hint when the site seems to want a login.
- **A quick action**: `--clipboard` reads the copied link at once, and
  `<link>` the one on the command line, for starting keepclip from a keyboard
  shortcut, a launcher or a mouse button.
- **A console version**, `keepclip-cli.exe`, with the same options.
- **The programs that do the work**, yt-dlp, ffmpeg and Deno, downloaded on
  the first run after asking, from their official releases, each checked
  against its published SHA-256 checksum. yt-dlp updates itself when its last
  check is more than a week old. The settings show the download folder and
  the version of each program.
- **Downloads**: an `.msi` that installs for your user only, without
  administrator rights, adds keepclip to the Start menu and `keepclip-cli` to
  your PATH, and when uninstalled removes the programs it downloaded and keeps
  your settings; and a portable `.zip`. Both come with `SHA256SUMS` and the
  licenses of the third-party code.

### Known limitations

- The files are not code-signed yet, so Windows SmartScreen ("Windows
  protected your PC") and some antivirus programs may warn about them. The
  README explains how to check a download.
- Videos that need a login do not download yet: keepclip cannot log in to
  sites.
- Sites change often, and a download from a site that changed may fail until
  yt-dlp catches up; keepclip updates yt-dlp once a week.
- The first run downloads about 150 MB of programs, which take about 310 MB of
  disk. ffmpeg and Deno are not updated after that.
- The window needs the WebView2 Runtime. Windows 11 has it, and Windows Update
  installs it on most Windows 10 computers; where it is missing, the window
  says where to get it, and the console version works without it.
- Long titles make long file names, cut at 120 characters.
- Windows only, and the interface is in English.

[unreleased]: https://github.com/iglesiaskun/keepclip/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/iglesiaskun/keepclip/releases/tag/v0.1.0
