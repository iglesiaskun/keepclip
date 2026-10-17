# keepclip

[![CI](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml/badge.svg)](https://github.com/iglesiaskun/keepclip/actions/workflows/ci.yml)

[Leia em português](README.pt-BR.md)

Save a video from a link you copied, without ad-filled download sites: copy
the link, open keepclip from the Start menu, pick a quality by its size, and
the file lands in your download folder. It works with the
[sites yt-dlp supports](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md),
and it closes when it is done: nothing keeps running in the background.

> **Download only what you have the right to keep:** your own videos, videos
> under a license that allows it, or videos whose owner lets you save them.
> Respect copyright and the terms of each site. keepclip is a tool; what you
> do with it is up to you.

keepclip is at its first version. It comes as a small window,
`keepclip.exe`, and as a console version, `keepclip-cli.exe`, with the same
options. What changes in each version is in the [changelog](CHANGELOG.md).

## How it works

1. Copy the link of a video.
2. Open keepclip from the Start menu. The link you copied is already in its
   field; or paste one.
3. Click **Read link**. keepclip asks yt-dlp what the link offers and shows
   the video, with its thumbnail, and the choices, with the size of each.
4. Click one, or press its number. keepclip shows the progress, saves the
   file and offers **Open folder**, **Play** and **Download another**. The
   window closes by itself five seconds later, unless the mouse is over it.

keepclip can also start from a keyboard shortcut, a launcher or a mouse
button and read the copied link at once: see
[Start from another app](#start-from-another-app).

## Install

Requirements:

- Windows 10 or 11, 64-bit.
- For the window, the WebView2 Runtime, the part of Windows that shows web
  pages inside apps. Windows 11 comes with it, and Windows Update has already
  installed it on most Windows 10 computers. Where it is missing,
  `keepclip.exe` says so, with the link to download it, for free:
  <https://developer.microsoft.com/microsoft-edge/webview2/>. The console
  version, `keepclip-cli.exe`, works without it.

To install:

1. Download the latest version from the
   [Releases page](https://github.com/iglesiaskun/keepclip/releases/latest):
   - **`keepclip-<version>-windows-x64.msi`** (recommended) installs keepclip
     for your user only, without administrator rights, into
     `%LOCALAPPDATA%\Programs\keepclip`. It adds keepclip to the Start menu
     and its folder to your PATH, so `keepclip-cli` works in any terminal you
     open afterwards. To update, run the newer `.msi`; a keepclip that is
     open closes first.
   - **`keepclip-<version>-windows-x64.zip`** runs without installing: unzip
     it anywhere and start `keepclip.exe`.
2. If Windows shows "Windows protected your PC", click **More info** and then
   **Run anyway**. The files are not code-signed yet, so SmartScreen warns
   about any new download from keepclip; the files are built on GitHub from
   this repository, and you can check them, as explained in
   [Check a download](#check-a-download).
3. Open **keepclip** from the Start menu.

### First run

keepclip asks where to save the downloads and keeps the answer in its
settings file.

Then it downloads, once, the free programs that do the work, into
`%LOCALAPPDATA%\keepclip\bin`. It asks before downloading (the window asks
for the folder and the programs on one screen), takes each one from its
official release on GitHub and checks it against the SHA-256 checksum
published with it:

| Program | What it does | From | License |
|---|---|---|---|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | reads the link and downloads the video | `yt-dlp/yt-dlp` | Unlicense |
| [ffmpeg](https://ffmpeg.org) | joins video and sound, makes MP3s | `yt-dlp/FFmpeg-Builds`, the builds made for yt-dlp | GPL |
| [Deno](https://deno.com) | runs the JavaScript some sites need | `denoland/deno` | MIT |

The download is about 150 MB, and the programs take about 310 MB of disk.
keepclip itself does not include or distribute any of them. Sites change
often, so keepclip updates yt-dlp (`yt-dlp -U`) when its last check is more
than 7 days old.

### Check a download

Each release lists the SHA-256 checksum of its files in `SHA256SUMS`.
Compare the line for your file with what PowerShell prints for it:

```powershell
Get-FileHash keepclip-0.1.0-windows-x64.msi
```

The files also carry a build attestation, which says they were built by this
repository's release workflow. With the [GitHub CLI](https://cli.github.com):

```powershell
gh attestation verify keepclip-0.1.0-windows-x64.msi --repo iglesiaskun/keepclip
```

### Uninstall

**Installed with the `.msi`**: remove keepclip in Settings > Apps >
Installed apps. That also deletes the programs it downloaded and the
window's own files (`%LOCALAPPDATA%\keepclip`), the parts of unfinished
downloads (`%TEMP%\keepclip`) and its folder in your PATH. The settings
stay in `%APPDATA%\keepclip`, for a later install; delete that folder to
remove everything.

**From the `.zip`**: delete its folder, then the folders
`%APPDATA%\keepclip` (settings) and `%LOCALAPPDATA%\keepclip` (the
downloaded programs and the window's own files), and `%TEMP%\keepclip` if
it is there (parts of unfinished downloads). To remove the name keepclip
registers for its notifications, delete the registry key
`HKEY_CURRENT_USER\Software\Classes\AppUserModelId\keepclip`.

Either way, your videos stay in the download folder.

## Using keepclip

Opened from the Start menu, keepclip shows a field for the link. A link you
have copied is already in it, and a link copied while keepclip is open goes
in when you come back to the window, unless you have typed in the field.
Nothing is read until you click **Read link** or press Enter. The gear next to the folder, at the bottom,
opens the settings: the download folder and the version of each program.

From a command line or a shortcut:

| Command | Does |
|---|---|
| `keepclip` | opens with the copied link in the field, ready to read |
| `keepclip --clipboard` | reads the link in the clipboard at once |
| `keepclip <link>` | reads that link at once |
| `keepclip --choose-folder` | picks the download folder again; alone, it opens the settings |
| `keepclip --dir <folder>` | keeps the settings file in that folder, handy for testing |

`keepclip-cli` takes the same options; without a link it uses the one in the
clipboard, or asks for one. In the window, the keys 1 to 9 pick a line and
Esc closes it.

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

**Files already in the folder.** When the folder already has a file with the
same name, such as the same video at another resolution or its MP3 from
before, keepclip asks before downloading whether to replace it. The answer
is no unless you choose **Replace** (in the console, type `y`). If you say
yes, the old file stays as it is until the new one is complete, so a failed
download loses nothing.

keepclip makes each file in `%TEMP%\keepclip` and moves it to your folder
when it is complete. Parts of a download that stopped halfway, with
**Cancel** or an error, stay there, and yt-dlp picks them up when you
download the same choice again.

**A link with several videos**, such as a post with more than one, shows a
list of them first (up to 20) and downloads the one you pick.

**When something goes wrong**, keepclip shows the reason yt-dlp gave, ready
to copy, with **Try again**. The console keeps its window open until you
press Enter.

**The notification.** When a download ends while the window is not in front,
keepclip shows a Windows notification; click it to open the folder. The
console always shows it.

### Start from another app

keepclip also works as a quick action: copy a link, press a key or a mouse
button, pick a quality, done. Any launcher, keyboard shortcut or mouse tool
that runs a program can start it with the link you copied:

```
"%LOCALAPPDATA%\Programs\keepclip\keepclip.exe" --clipboard
```

(With the `.zip`, use the folder you unzipped it to.) The window opens at
once, in the middle of the screen the mouse is on, already reading the link,
and with the focus, so the number keys work without a click. When the
clipboard has no link, it shows the field to paste one, and reads a link
copied while it waits.

`keepclip-cli.exe --clipboard` does the same in a console. When it has the
console window to itself, it closes it two seconds after the download; the
notification stays in the Windows notification center. If Windows does not
show the notification, the console waits for Enter instead.

## Configuration

The settings file is `%APPDATA%\keepclip\config.toml`. keepclip writes it
when you pick the download folder (**Change**, at the bottom of the window,
or in the settings, picks another), and you can edit it:

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
- **In English** only.
- **Windows only** for now. The logic in `core/` has no Windows code; on
  other systems keepclip expects `yt-dlp`, `ffmpeg` and `deno` in the PATH,
  but it is not tested there.
- **Long titles**, such as posts full of hashtags and emoji, make long file
  names, cut at 120 characters.
- **Not code-signed yet.** Windows SmartScreen and some antivirus programs
  warn about new programs without a signature and a reputation. Code signing
  is planned; until then, [Check a download](#check-a-download) shows how to
  confirm a file came from this repository.

## Privacy

keepclip has no telemetry. It connects to GitHub to download and update its
programs, through yt-dlp to the sites of the links you give it, and, for the
thumbnail in the window, to the address of the picture yt-dlp reports.
Nothing else.

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
`target\release\keepclip-cli.exe` (the console). The interface of the
window, in `window/ui`, is plain HTML, CSS and JavaScript: there is no Node,
npm or other build step.

The release files come from `packaging/package.ps1`, which the release
workflow runs; the comment at its top explains how to run it locally. It
needs [WiX Toolset](https://wixtoolset.org) 5 for the `.msi` and
[cargo-about](https://github.com/EmbarkStudios/cargo-about) for the list of
third-party licenses.

## Project layout

```
core/     library (keepclip-core): pure, testable logic with no Windows
          code: reads what yt-dlp reports, builds the menu, the download
          arguments and the settings file, and follows the progress
system/   library (keepclip-system): what both programs share: runs yt-dlp,
          downloads and updates yt-dlp, ffmpeg and Deno, the settings file,
          the clipboard and the notification
window/   the window (keepclip.exe), on Tauri 2, with its interface in ui/
cli/      the console version (keepclip-cli.exe)
packaging/ the installer (WiX), the portable .zip and the list of
          third-party licenses, made by package.ps1
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

The icons of the window, in [`window/ui/icons`](window/ui/icons), come from
[Lucide](https://lucide.dev) and are under the ISC license
([LICENSE-LUCIDE](window/ui/icons/LICENSE-LUCIDE)).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
