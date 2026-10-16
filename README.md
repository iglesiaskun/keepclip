# keepclip

[![CI](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml/badge.svg)](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml)

[Leia em português](README.pt-BR.md)

Save a video from a link you copied, without ad-filled download sites: copy
the link, start keepclip, pick a quality by its size, and the file lands in
your download folder. It works with the
[sites yt-dlp supports](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md),
and it closes when it is done: nothing keeps running in the background.

> **Download only what you have the right to keep:** your own videos, videos
> under a license that allows it, or videos whose owner lets you save them.
> Respect copyright and the terms of each site. keepclip is a tool; what you
> do with it is up to you.

keepclip is at its first version, which runs in a console window. What
changes in each version is in the [changelog](CHANGELOG.md).

## How it works

1. Copy the link of a video.
2. Start keepclip with `--clipboard`, from a shortcut, a launcher or a mouse
   action (see [Start from another app](#start-from-another-app)).
3. keepclip asks yt-dlp what the link offers and shows the choices, with the
   size of each:

   ```
   Sample video | Sample channel | 10:35

      1) 1080p 60 fps      255.5 MB
      2) 720p 60 fps       153.3 MB
      3) 480p              36.7 MB
      4) Audio only (MP3)  ~14.5 MB

   Choose (1-4, Enter = 1, 0 = cancel):
   ```

4. Pick one. keepclip shows the progress, saves the file and shows a Windows
   notification; click it to open the folder.

## Install

There are no ready-made downloads yet: build keepclip from source, as
explained in [Building from source](#building-from-source). It needs Windows
10 or 11, 64-bit.

### First run

keepclip asks where to save the downloads and keeps the answer in its
settings file.

Then it downloads, once, the free programs that do the work, into
`%LOCALAPPDATA%\keepclip\bin`. It asks before downloading, takes each one
from its official release on GitHub and checks it against the SHA-256
checksum published with it:

| Program | What it does | From | License |
|---|---|---|---|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | reads the link and downloads the video | `yt-dlp/yt-dlp` | Unlicense |
| [ffmpeg](https://ffmpeg.org) | joins video and sound, makes MP3s | `yt-dlp/FFmpeg-Builds`, the builds made for yt-dlp | GPL |
| [Deno](https://deno.com) | runs the JavaScript some sites need | `denoland/deno` | MIT |

The download is about 150 MB, and the programs take about 310 MB of disk.
keepclip itself does not include or distribute any of them. Sites change
often, so keepclip updates yt-dlp (`yt-dlp -U`) when its last check is more
than 7 days old.

### Uninstall

Delete `keepclip.exe`, then the folders `%APPDATA%\keepclip` (settings) and
`%LOCALAPPDATA%\keepclip` (the downloaded programs), and `%TEMP%\keepclip`
if it is there (parts of unfinished downloads). To remove the name
keepclip registers for its notifications, delete the registry key
`HKEY_CURRENT_USER\Software\Classes\AppUserModelId\keepclip`. Your videos stay
in the download folder.

## Using keepclip

```bash
keepclip --clipboard
```

downloads the link in the clipboard. The other ways to start it:

| Command | Does |
|---|---|
| `keepclip <link>` | downloads that link |
| `keepclip` | uses the link in the clipboard, or asks you to paste one |
| `keepclip --choose-folder` | picks the download folder again |
| `keepclip --dir <folder>` | keeps the settings file in that folder, handy for testing |

**The choices.** There is one line per resolution, from the highest down,
and then **Audio only (MP3)**. For each resolution keepclip takes the video
that plays almost everywhere, H.264 with AAC sound in an MP4 file, when the
site has it. Above the highest resolution in H.264, as with some videos above
1080p, it takes the best video there is, which may play in fewer places;
below it, resolutions without H.264 are left out. Files are MP4, or MKV when
the video and the sound do not fit in an MP4.

**The MP3** is 192 kbps, made from the video's own sound, so its size is
known in advance from the length.

**Sizes** marked "~" are estimates, by the site or from the bitrate and the
length; "size unknown" means the site gave neither.

**File names** follow `<uploader> - <title> [<video id>].<ext>`, with the
channel when the site leaves the uploader empty, characters Windows does not
allow replaced, and a cut at 120 characters. Titles come as the site gives
them, emoji and hashtags included.

**Files already in the folder.** A video whose file is already there is not
downloaded again, even at another resolution, since the name is the same:
move or rename the old one first. An MP3 downloaded again replaces the old
one. Parts of a download that stopped halfway stay in `%TEMP%\keepclip`, and
yt-dlp picks them up when you download the same choice again.

**A link with several videos**, such as a post with more than one, shows a
list of them first (up to 20) and downloads the one you pick.

**When something goes wrong**, keepclip shows the reason yt-dlp gave and
keeps the window open until you press Enter.

### Start from another app

keepclip is made to be started by something else and close when it is done.
Any launcher, keyboard shortcut or mouse action that runs a program can start
it with the link you copied:

```
"C:\path\to\keepclip.exe" --clipboard
```

When it has the console window to itself, keepclip closes it two seconds
after the download; the notification stays in the Windows notification
center. If Windows does not show the notification, the window waits for
Enter instead.

## Configuration

The settings file is `%APPDATA%\keepclip\config.toml`. keepclip writes it
when you pick the download folder, and you can edit it:

```toml
version = 1

# Folder where downloads are saved.
download_dir = 'C:\Users\you\Downloads'
```

`version = 1` is the format of the file. If a future version changes the
format, keepclip uses it to update older files. A file without it counts as
version 1.

## Known limitations

- **Sites that need a login** do not work yet: keepclip cannot log in for
  you. Public videos work.
- **Console only**, in English. A window comes in a later version.
- **Windows only** for now. The logic in `core/` has no Windows code; on
  other systems keepclip expects `yt-dlp`, `ffmpeg` and `deno` in the PATH,
  but it is not tested there.
- **Long titles**, such as posts full of hashtags and emoji, make long file
  names, cut at 120 characters.

## Privacy

keepclip has no telemetry. It connects to GitHub to download and update its
programs, and through yt-dlp to the sites of the links you give it. Nothing
else.

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

Then run `target\release\keepclip.exe`.

## Project layout

```
core/   library (keepclip-core): pure, testable logic with no Windows code:
        reads what yt-dlp reports, builds the menu, the download arguments
        and the settings file, and follows the progress
cli/    the program (keepclip.exe): console, clipboard, notification, and
        the download and update of yt-dlp, ffmpeg and Deno
```

## Contributing

Bug reports, ideas and pull requests are welcome. [CONTRIBUTING.md](CONTRIBUTING.md)
explains how changes get in. Please report security problems privately, as
described in [SECURITY.md](SECURITY.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

The programs keepclip downloads on the first run (yt-dlp, ffmpeg, Deno) are
not part of keepclip and keep their own licenses.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
