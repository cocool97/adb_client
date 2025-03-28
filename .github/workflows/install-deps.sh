#!/bin/bash
echo "Running prebuild"
sudo apt-get -y install libmagic-dev libudev-dev
rm -Rf /opt/osxcross/tarballs/*
curl -Lo /opt/osxcross/tarballs/MacOSX15.2.sdk.tar.xz "https://github.com/joseluisq/macosx-sdks/releases/download/15.2/MacOSX15.2.sdk.tar.xz"
cd /opt/osxcross && UNATTENDED=yes OSX_VERSION_MIN=10.12 ./build.sh
echo "Completed prebuild"
