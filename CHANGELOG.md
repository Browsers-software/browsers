# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2024-01-27

### Added
- Add Settings dialog for configuring opening rules.
- Add Edge (stable, dev, beta, canary) profile support for Windows. Fixes #88

## [0.4.5] - 2024-01-11

### Fixed
- Disable keyboard shortcuts if About dialog is focused

## [0.4.4] - 2023-12-29

### Fixed
- Windows: Fix various installer issues. Thanks to @Erquint for thorough report #116
- Windows: Don't require "Microsoft Visual C++ Redistributable" to be installed

### Added
- Windows: Add uninstaller to Installed Apps

## [0.4.3] - 2023-10-24

### Fixed
- macOS: Exit on lost focus when application looses focus not the window. #95

## [0.4.2] - 2023-09-25

### Fixed
- Linux: Fix opening Browsers on top of a Snap application window Contribution by @tommoyer. #87
- Linux: Handle more errors when loading app icons. #90
- Linux: Fix initial position in Wayland
- Windows: Fix shortcut in system-wide installation

### Added
- Support deeplink for a Slack canvas
- Linux: uninstall.sh script in the archive

## [0.4.1] - 2023-08-01

### Fixed
- Fix all urls when using Firefox containers #73. Contribution by @blandir.
- macOS: Don't prompt to be default browser on launch. There is an option to set it via default via options menu. #76
- macOS: Workaround for Safari bug where Safari launches Browsers on hard launch. #79

### Added
- Beta configuration to enable for Browsers to quit when clicking outside the Browsers dialog. #77

## [0.4.0] - 2023-07-15

### Fixed
- Windows: Fix install script
- Firefox Containers: Fix urls with `&` and `+` #73

### Added
- Linux: provide rpm package

## [0.3.9] - 2023-06-19

### Fixed:
- Opening browser in private in Linux and Windows when shift + mouse click
- Opening browser in private mode with shift + <number key>

## [0.3.8] - 2023-06-17

### Added
- Linux: provide deb package

## [0.3.7] - 2023-06-16

### Added
- Support Slack desktop app for `<team>.slack.com` urls
- Allow opening browser by numerical key 1, 2, ..., 9, 0

### Fixed:
- Windows: Fix install script
- Windows: Fix shortcut when installing for all users

## [0.3.6] - 2023-05-31

### Added

- Allow installing under system path in Linux (/usr/local/bin/) and Windows (%ProgramFiles%)

### Fixed:

- Make UI a big higher. Might need to reduce it only in Windows, as Windows seems to fit more
- Stable ordering of profiles when refreshing

## [0.3.5]- 2023-05-28

### Added

- Linux: support armv7l (mostly 32-bit raspberry pi)
- Windows: initial support for non-browser apps (Spotify, Linear, Zoom, etc)
- macOS/Windows: Add support for Workflowy app

### Security

- Get rid of `chrono` library with outdated `time` dependency and use our own `rolling-file-rs` fork

### Changed

- macOS: show always on top

### Fixed

- Linux: fix making window visible when clicking on another link
- Windows: fix running install.bat straight from the zip file
- Fix order of apps when restoring a profile
- Fix 3-dot menu button shifting/disappearing when clicking on another link
- Linux: fix window size in LXDE (Openbox)

## [0.3.4] - 2023-05-20

### Changed

- Wildcards in URL rules #18
- Optional default browser #18

### Fixed

- Linux: Opening browsers with multiargument Exec line in .desktop file #32

## [0.3.3] - 2023-05-14

### Fixed

- Windows: show also browsers installed to current user only

## [0.3.2] - 2023-05-14

### Fixed

- Fix opening link in Windows

## [0.3.1] - 2023-05-14

### Changed

- Use cached Chrome profile images instead of downloading from the internet
- Support Chrome profile images picked from image choice
- Use symlink for linux binary

### Added

- Initial Windows support

## [0.3.0] - 2023-05-02

### Fixed

- Allow picking Browsers as a default browser in XFCE

## [0.2.9] - 2023-05-01

### Fixed

- Run only single instance of Browsers. Fixes #15

## [0.2.8] - 2023-04-30

### Fixed

- Linux: Convert .svg icons and cache all used icons as png.

## [0.2.7] - 2023-04-28

### Fixed

- Linux: Use the full binary path in software.Browsers.desktop, because many distros don't have $HOME/.local/bin in PATH
- Linux: Include icon in About dialog and translations
- Linux: Move config file from $HOME/.local/share/software.Browsers/config.json to
  $HOME/.config/software.Browsers/config.json

## [0.2.6] - 2023-04-27

### Fixed

- Recognize chromium.desktop (e.g MX Linux) as Chrome based browser

## [0.2.5] - 2023-04-25

### Fixed

- Improve installation in Linux (auto-creates ~/.local/bin/ directory)
- Don't crash if Firefox is detected, but profiles.ini is missing

## [0.2.4] - 2023-04-25

### Fixed

- Recognize firefox.desktop (e.g KDE Neon) and firefox-esr.desktop (e.g Raspberry Pi OS) as Firefox
- Recognize chromium-browser.desktop (e.g Raspberry Pi OS) as Chrome
- Remove openssl system dependency (ssl is used to verify https when fetching profile images)

## [0.2.3] - 2023-03-29

### Fixed

- Don't crash if a browser doesn't have an icon #12

## [0.2.2] - 2023-03-14

### Fixed

- Set correct version in macOS Browsers.app

## [0.2.1] - 2023-02-15

### Fixed

- Show dialog when clicking a link when About dialog is also open

## [0.2.0] - 2023-02-14

### Fixed

- Update dialog position when clicking a link when old dialog is still open

## [0.1.0] - 2023-01-17

Initial Release

[unreleased]: https://github.com/Browsers-software/browsers/compare/0.5.0...HEAD
[0.5.0]: https://github.com/Browsers-software/browsers/releases/tag/0.5.0
[0.4.5]: https://github.com/Browsers-software/browsers/releases/tag/0.4.5
[0.4.4]: https://github.com/Browsers-software/browsers/releases/tag/0.4.4
[0.4.3]: https://github.com/Browsers-software/browsers/releases/tag/0.4.3
[0.4.2]: https://github.com/Browsers-software/browsers/releases/tag/0.4.2
[0.4.1]: https://github.com/Browsers-software/browsers/releases/tag/0.4.1
[0.4.0]: https://github.com/Browsers-software/browsers/releases/tag/0.4.0
[0.3.9]: https://github.com/Browsers-software/browsers/releases/tag/0.3.9
[0.3.8]: https://github.com/Browsers-software/browsers/releases/tag/0.3.8
[0.3.7]: https://github.com/Browsers-software/browsers/releases/tag/0.3.7
[0.3.6]: https://github.com/Browsers-software/browsers/releases/tag/0.3.6
[0.3.5]: https://github.com/Browsers-software/browsers/releases/tag/0.3.5
[0.3.4]: https://github.com/Browsers-software/browsers/releases/tag/0.3.4
[0.3.3]: https://github.com/Browsers-software/browsers/releases/tag/0.3.3
[0.3.2]: https://github.com/Browsers-software/browsers/releases/tag/0.3.2
[0.3.1]: https://github.com/Browsers-software/browsers/releases/tag/0.3.1
[0.3.0]: https://github.com/Browsers-software/browsers/releases/tag/0.3.0
[0.2.9]: https://github.com/Browsers-software/browsers/releases/tag/0.2.9
[0.2.8]: https://github.com/Browsers-software/browsers/releases/tag/0.2.8
[0.2.7]: https://github.com/Browsers-software/browsers/releases/tag/0.2.7
[0.2.6]: https://github.com/Browsers-software/browsers/releases/tag/0.2.6
[0.2.5]: https://github.com/Browsers-software/browsers/releases/tag/0.2.5
[0.2.4]: https://github.com/Browsers-software/browsers/releases/tag/0.2.4
[0.2.3]: https://github.com/Browsers-software/browsers/releases/tag/0.2.3
[0.2.2]: https://github.com/Browsers-software/browsers/releases/tag/0.2.2
[0.2.1]: https://github.com/Browsers-software/browsers/releases/tag/0.2.1
[0.2.0]: https://github.com/Browsers-software/browsers/releases/tag/0.2.0
[0.1.0]: https://github.com/Browsers-software/browsers/releases/tag/0.1.0-rc25