#!/usr/bin/env bash

# exit when any command fails
set -e

get_file_size() {
  local file_path="$1"
  local file_size
  file_size=$(stat -f%z "$file_path")
  # on Linux
  #FILE_SIZE=$(stat -c%s "$FILE_PATH")
  # on linux:
  #stat --printf="%s" file.any
  echo "$file_size"
}

get_file_signature() {
  local file_path="$1"
  local signature_local_path
  signature_local_path="$file_path.sig"

  local file_signature

  # print only 2nd line and quit
  file_signature=$(sed -n '2{p;q;}' "$signature_local_path")

  echo "$file_signature"
}

create_enclosure_macos() {
  local file_name="browsers_mac.tar.gz"
  local file_local_path="../../target/universal-apple-darwin/release/$file_name"
  local file_size
  file_size=$(get_file_size "$file_local_path")

  local file_signature
  file_signature=$(get_file_signature "$file_local_path")

  local sparkle_os="macos"
  local enclosure_url="https://github.com/Browsers-software/browsers/releases/download/$version/$file_name"

  cat << EOM
    <enclosure url="$enclosure_url"
               sparkle:edSignature="$file_signature"
               sparkle:os="$sparkle_os"
               length="$file_size"
               type="application/octet-stream"/>
EOM
}

create_enclosure_linux() {
  local file_name="browsers_linux.tar.gz"
  local file_local_path="../../target/universal-unknown-linux/release/$file_name"
  local file_size
  file_size=$(get_file_size "$file_local_path")
  local file_signature
  file_signature=$(get_file_signature "$file_local_path")

  local sparkle_os="linux"
  local enclosure_url="https://github.com/Browsers-software/browsers/releases/download/$version/$file_name"

  cat << EOM
    <enclosure url="$enclosure_url"
               sparkle:edSignature="$file_signature"
               sparkle:os="$sparkle_os"
               length="$file_size"
               type="application/octet-stream"/>
EOM
}

# ./tools/extract_release_notes.sh ${{ github.ref_name }} < CHANGELOG.md > ${{ github.workspace }}-RELEASE_NOTES.md


create_item() {
  local version="$1" # "x.y.z"
  local description="$2" # changelog
  local pubDate="$3" # "14 Mar 2023 12:00:00 Z"
  local title="Version $version"
  local releaseNotesLink="https://github.com/Browsers-software/browsers/releases/tag/$version"
  local link="$releaseNotesLink"

  local enclosure_macos
  enclosure_macos=$(create_enclosure_macos)
  local enclosure_linux
  enclosure_linux=""
  #enclosure_linux=$(create_enclosure_linux)

  cat << EOM
<item>
    <title>$title</title>
    <description><![CDATA[
      $description
    ]]></description>
    <pubDate>$pubDate</pubDate>
    <link>$link</link>
    <sparkle:version>$version</sparkle:version>
    <sparkle:releaseNotesLink>$releaseNotesLink</sparkle:releaseNotesLink>
$enclosure_macos
$enclosure_linux
</item>
EOM
}

datetime=$(LC_ALL=C TZ="UTC" date +"%d %b %Y %H:%M:%S Z")
create_item "0.2.2" "Version description" "$datetime"