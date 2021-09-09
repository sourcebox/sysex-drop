# SysEx Drop

## About

**SysEx Drop** is a cross-platform utility for sending SysEx files to MIDI devices via a simple drag-and-drop GUI.

![Screenshot](screenshot.png)

## Usage

Just drop a file onto the application window and press the **Start** button. The SysEx data from the file is then sent in individual packages. The delay between the packets is adjustable from 0 to 500ms with a default of 20ms. If the receiving device does not recognize the data correctly, try to increase the delay setting.

## Build Requirements

- [Rust toolchain](https://www.rust-lang.org/)

On Linux, a couple of additional dependencies must be installed:

    - libxcb-render0-dev
    - libxcb-shape0-dev
    - libxcb-xfixes0-dev
    - libxkbcommon-dev

### Mac Application Bundle (optional)

To build a macOS application bundle, additional dependencies must be installed:

- [cargo-bundle](https://github.com/burtonageo/cargo-bundle)
- [Python3](https://python.org) (any recent version should work)

Run `./build-mac-bundle.sh` from the project directory. Make sure the script has executable permissions.
The bundle will be created in the `./target/release/bundle/osx` directory.

### Linux AppImage (optional)

To build an AppImage for Linux, additional dependencies must be installed:

- [linuxdeploy](https://github.com/linuxdeploy/linuxdeploy)
- [linuxdeploy-plugin-appimage](https://github.com/linuxdeploy/linuxdeploy-plugin-appimage)

Run `./build-linux-appimage.sh` from the project directory. Make sure the script has executable permissions.
The AppImage will be created in the `./target/release/appimage` directory.

## License

Published under the MIT license.

Author: Oliver Rockstedt <info@sourcebox.de>

## Donations

If you like to support my work, you can [buy me a coffee.](https://www.buymeacoffee.com/sourcebox)

<a href="https://www.buymeacoffee.com/sourcebox" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174"></a>
