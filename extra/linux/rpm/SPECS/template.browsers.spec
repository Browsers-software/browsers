Name:           browsers
Version:        €Version€
Release:        1
Summary:        Context menu displaying a list of apps when clicking on a web link
License:        MIT AND Apache-2.0
Requires:       gtk3, pango

%define _rpmfilename %%{NAME}.%%{ARCH}.rpm
%define _bindir /usr/bin
%define _datadir /usr/share
%define _arch €Architecture€
%define _tree %{_topdir}/tree

# https://groups.google.com/g/linux.redhat.rpm/c/TKz6JZHK0ck
%define __os_install_post %{nil}

# also see https://docs.fedoraproject.org/en-US/packaging-guidelines/#_tags_and_sections
# also see https://jfearn.fedorapeople.org/fdocs/en-US/Fedora_Draft_Documentation/0.1/
#          html/Packagers_Guide/chap-Packagers_Guide-Spec_File_Reference-Preamble.html

%description
Browsers is an intuitive context menu that pops up
when you press a link in an app other than a web browser.

%prep
# we have no source, so nothing here

%install
# Don't execute strip command on binary, because that would work only
# on binaries with same architecture as the runner
# but we are running it in x86_64 on github ci runner
#export DONT_STRIP=1

ls -altrh %{_tree}
ls -altrh %{_tree}/usr/bin
ls -altrh %{_tree}/usr/share

mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/software.Browsers/bin
mkdir -p %{buildroot}%{_datadir}/software.Browsers/resources/i18n/en-US
mkdir -p %{buildroot}%{_datadir}/software.Browsers/resources/icons/512x512
mkdir -p %{buildroot}%{_datadir}/software.Browsers/resources/repository
mkdir -p %{buildroot}%{_datadir}/applications/xfce4/helpers
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/512x512/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/256x256/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/128x128/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/64x64/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/32x32/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/16x16/apps

# Copy all files directly to buildroot
# + mkdir -p /.../BUILDROOT/browsers-0.3.9-1.arm64/usr/share/applications
# -a is same as -dR --preserve=all
# For us it's important that it:
#  - preserves symlinks
#  - copies recursively
#cp -a %{_tree}/ %{buildroot}/

cp -a %{_tree}%{_datadir}/applications/software.Browsers.desktop %{buildroot}%{_datadir}/applications/software.Browsers.desktop
cp -a %{_tree}%{_datadir}/applications/xfce4/helpers/software.Browsers.desktop %{buildroot}%{_datadir}/applications/xfce4/helpers/software.Browsers.desktop

for size in 16 32 64 128 256 512; do
    cp -a %{_tree}%{_datadir}/icons/hicolor/${size}x${size}/apps/software.Browsers.png %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/software.Browsers.png
done

cp -a %{_tree}%{_datadir}/software.Browsers/bin/browsers %{buildroot}%{_datadir}/software.Browsers/bin/browsers
cp -a %{_tree}%{_datadir}/software.Browsers/resources/i18n/en-US/builtin.ftl %{buildroot}%{_datadir}/software.Browsers/resources/i18n/en-US/builtin.ftl
cp -a %{_tree}%{_datadir}/software.Browsers/resources/icons/512x512/software.Browsers.png %{buildroot}%{_datadir}/software.Browsers/resources/icons/512x512/software.Browsers.png
cp -a %{_tree}%{_datadir}/software.Browsers/resources/repository/application-repository.toml %{buildroot}%{_datadir}/software.Browsers/resources/repository/application-repository.toml

cp -a %{_tree}%{_bindir}/browsers %{buildroot}%{_bindir}/browsers

%files
# set default user and group as root
%defattr(-,root,root,-)

# _bindir: /usr/bin
# _datadir: /usr/share

%{_bindir}/browsers
%{_datadir}/applications/software.Browsers.desktop
%{_datadir}/applications/xfce4/helpers/software.Browsers.desktop
%{_datadir}/icons/hicolor/*/apps/software.Browsers.png
%{_datadir}/software.Browsers/bin/browsers
%{_datadir}/software.Browsers/resources/i18n/en-US/builtin.ftl
%{_datadir}/software.Browsers/resources/icons/512x512/software.Browsers.png
%{_datadir}/software.Browsers/resources/repository/application-repository.toml

%changelog
# let's skip this for now
