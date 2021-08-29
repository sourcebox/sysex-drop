#!/bin/bash

cargo build --release

rm -rf target/release/appimage/*

linuxdeploy-x86_64.AppImage \
    --executable ./target/release/sysex-drop \
    --desktop-file ./tools/sysex-drop.desktop \
    --icon-file ./icons/16x16/sysex-drop.png \
    --icon-file ./icons/32x32/sysex-drop.png \
    --icon-file ./icons/64x64/sysex-drop.png \
    --icon-file ./icons/128x128/sysex-drop.png \
    --icon-file ./icons/256x256/sysex-drop.png \
    --icon-file ./icons/sysex-drop.svg \
    --appdir ./target/release/appimage/AppDir \
    --output appimage

echo "Moving appimage to target directory"
mv *.AppImage ./target/release/appimage/
