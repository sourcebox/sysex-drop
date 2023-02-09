# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Updated `eframe` dependency to `0.21`

### Fixed

- Show error message instead of panic when MIDI port is unavailable.
- Restore window position correctly on Linux and Windows.
- Save persistent settings on macOS when Cmd-Q is pressed.

## [1.2.0] - 2022-04-25

### Added

- Auto-Start feature.
- Frame rate limit to reduce CPU load.
- Strip symbols from release builds to reduce binary size.

### Changed

- Set window size on startup using an alternative method.

## [1.1.0] - 2022-01-26

### Added

- Support for Standard MIDI Files (SMF).
- Automatic device rescan on MIDI output ports change.

### Changed

- Updated `eframe` dependency to `0.16`
- Updated `simple_logger` dependency to `2.1`. Now using UTC timestamps.

### Removed

- Rescan button. Device changes are now detected automatically.

## [1.0.0] - 2021-10-31

First stable release.

## [0.1.0] - No date specified

Development release for initial testing.
