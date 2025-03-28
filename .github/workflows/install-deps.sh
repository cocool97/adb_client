#!/bin/bash
sudo apt-get -y install libmagic-dev
rm -Rf /opt/osxcross/tarballs/*
curl -Lo /opt/osxcross/tarballs/MacOSX15.2.sdk.tar.xz "https://github.com/joseluisq/macosx-sdks/releases/download/15.2/MacOSX15.2.sdk.tar.xz"
