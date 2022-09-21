https://packages.debian.org/sid/desktop-file-utils

# validate .desktop file is OK
#desktop-file-validate software.Browsers.desktop

# validate and copy .desktop file to /usr/share/applications/
./setup-handler.sh

#sudo desktop-file-install software.Browsers.desktop

# refresh db
update-desktop-database
