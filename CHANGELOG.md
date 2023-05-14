# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/Browsers-software/browsers/compare/0.3.3...HEAD
[0.3.0]: https://github.com/Browsers-software/browsers/releases/tag/0.3.3
[0.3.0]: https://github.com/Browsers-software/browsers/releases/tag/0.3.2
[0.3.0]: https://github.com/Browsers-software/browsers/releases/tag/0.3.1
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