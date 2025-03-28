#!/bin/bash
echo "Running prebuild"
apt-get -y install libmagic-dev libudev-dev musl-tools
rm -Rf /opt/osxcross/tarballs/*
curl -Lo /opt/osxcross/tarballs/MacOSX14.0.sdk.tar.xz "https://github.com/joseluisq/macosx-sdks/releases/download/14.0/MacOSX14.0.sdk.tar.xz"
cd /opt/osxcross && UNATTENDED=yes OSX_VERSION_MIN=10.12 ./build.sh
echo $SDK_VERSION
echo "Completed prebuild"
