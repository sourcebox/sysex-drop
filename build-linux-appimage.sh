#!/bin/bash

cargo build --release

rm -rf target/release/appimage/*

linuxdeploy-x86_64.AppImage \
    --executable ./target/release/sysex-drop \
    --desktop-file ./tools/sysex-drop.desktop \
    --icon-file ./tools/sysex-drop.png \
    --appdir ./target/release/appimage/AppDir \
    --output appimage

echo "Moving appimage to target directory"
mv *.AppImage ./target/release/appimage/
