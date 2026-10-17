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

- **A small window**, `keepclip.exe`: it opens in the middle of the screen
  the mouse is on, shows the video with its thumbnail, title, author and
  length, and one line per choice; a click, or the number key of the line,
  downloads it. It follows the light or dark mode of Windows.
- **A console version**, `keepclip-cli.exe`, with the same options.
- **Download from a copied link**: `--clipboard` takes the link from the
  clipboard, `<link>` takes it from the command line, and with neither the
  clipboard is used too; when it has no link, the window shows a field to
  paste one, and the console asks for it.
- **A menu with sizes**: one line per resolution, with the video that plays
  almost everywhere (H.264 and AAC in MP4) where the site has it, and the size
  of each choice: exact, estimated (~) by the site or from the bitrate and the
  length, or unknown.
- **Audio only**: saves the video's own sound as a 192 kbps MP3.
- **Progress** of each part of the download, also on the window's title and
  on its taskbar button. Cancel stops yt-dlp and keeps the parts: picking the
  same choice again picks up where it stopped. Once the file is saved, the
  window offers Open folder and Play and closes by itself after 5 seconds,
  unless the mouse is over it. A Windows notification, which opens the folder
  when clicked, comes when the window is not in front, and always from the
  console, which closes by itself afterwards or waits for Enter after an
  error.
- **Download folder** chosen on the first run and kept in
  `%APPDATA%\keepclip\config.toml`; Change, at the bottom of the window, and
  `--choose-folder` change it. In the window, `--choose-folder` alone opens
  the settings, with the folder and the version of each program.
- **Files with the same name**, such as the same video at another resolution,
  are replaced only if you say so when asked, and only once the new file is
  complete.
- **Programs downloaded on the first run**, after asking: yt-dlp, ffmpeg and
  Deno, from their official releases, each checked against its published
  SHA-256 checksum. yt-dlp updates itself when its last check is more than a
  week old. The window asks for the folder and the programs on one screen,
  and shows the progress of each program.
- **Links with several videos** show a list to pick one from.
- **Errors** show what yt-dlp said, ready to copy, with a hint when the site
  seems to want a login.

### Known limitations

- Sites that need a login do not work yet.
- In English, on Windows only.
- The window needs the WebView2 Runtime, which comes with Windows 11 and with
  the updates of Windows 10.

[unreleased]: https://github.com/iglesiaskun/keepclip/commits/main
