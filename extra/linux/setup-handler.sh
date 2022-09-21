#!/bin/sh

#https://packages.debian.org/sid/desktop-file-utils
#https://wiki.archlinux.org/title/desktop_entries#Installation

# validate .desktop file is OK
#desktop-file-validate software.Browsers.desktop

# copy .desktop file to /usr/share/applications/ if with sudo
#sudo
desktop-file-install --dir=$HOME/.local/share/applications/ --rebuild-mime-info-cache ./dist/software.Browsers.desktop

# refresh db
#update-desktop-database

update-desktop-database $HOME/.local/share/applications

# also interesting https://archlinux.org/packages/community/any/dex/
