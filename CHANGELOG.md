# Changelog

All notable changes to keepclip are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before 1.0, a
minor version (0.2.0, 0.3.0) may change the settings file format; when that
happens, keepclip updates older files by itself and this file says what
changed.

## [Unreleased]

### Added

- **Download from a copied link**: `keepclip --clipboard` takes the link from
  the clipboard, `keepclip <link>` takes it from the command line, and
  `keepclip` alone uses the clipboard or asks for a link.
- **A menu with sizes**: one line per resolution, with the video that plays
  almost everywhere (H.264 and AAC in MP4) where the site has it, and the size
  of each choice: exact, estimated (~) by the site or from the bitrate and the
  length, or unknown.
- **Audio only**: saves the video's own sound as a 192 kbps MP3.
- **Progress** of each part of the download, then a Windows notification that
  opens the folder when clicked. The window closes by itself afterwards, or
  waits for Enter after an error.
- **Download folder** chosen on the first run and kept in
  `%APPDATA%\keepclip\config.toml`; `--choose-folder` changes it.
- **Files with the same name**, such as the same video at another resolution,
  are replaced only if you say so when asked, and only once the new file is
  complete.
- **Programs downloaded on the first run**, after asking: yt-dlp, ffmpeg and
  Deno, from their official releases, each checked against its published
  SHA-256 checksum. yt-dlp updates itself when its last check is more than a
  week old.
- **Links with several videos** show a list to pick one from.

### Known limitations

- Sites that need a login do not work yet.
- Console only, in English, on Windows.

[unreleased]: https://github.com/iglesiaskun/keepclip/commits/main
