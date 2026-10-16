# Security policy

## Supported versions

Only the latest release gets security fixes.

| Version | Supported |
|---|---|
| Latest release | Yes |
| Older releases | No |

## Reporting a vulnerability

Please do not report security problems in public issues.

Report them privately on GitHub instead: open the **Security** tab of the
repository and click **Report a vulnerability**
([direct link](https://github.com/iglesiaskun/keepclip/security/advisories/new)). Only
the maintainers can see the report.

Please include:

- what the problem is and what someone could do with it;
- the steps to reproduce it;
- the keepclip version and the Windows version;
- any idea you have for a fix.

keepclip is maintained in spare time. We aim to answer within 7 days. Once
the problem is confirmed, we work on the fix with you, publish a release and a
security advisory, and credit you, unless you prefer to stay anonymous.

## How keepclip trusts the programs it downloads

On the first run keepclip downloads yt-dlp, ffmpeg and Deno over HTTPS from
their official releases on GitHub, and checks each file against the SHA-256
checksum published in the same release before using it. This catches a
corrupted or replaced download; it cannot catch a release that was itself
compromised at its source. Updates of yt-dlp go through `yt-dlp -U`, which
does its own checks.

## Scope

keepclip runs with the rights of the person who starts it. It reads the
clipboard, runs yt-dlp with the link it finds, and writes files into the
download folder and its own folders. Reports like these are welcome:

- a link, a web page or a site's answer that makes keepclip or the programs
  it runs do something other than download that video: run other programs,
  read files, or write outside the download folder;
- a way to make keepclip install a program that does not match the published
  checksum, or that comes from somewhere else;
- a release file that does not match the source code, or a weakness in the
  release workflow.

Out of scope: problems in yt-dlp, ffmpeg or Deno themselves, which go to
those projects, and what a downloaded video contains.
