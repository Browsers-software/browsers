#!/usr/bin/env bash

PROJECT_ROOT="../../.."
MACOS_RELEASE_DIR="$PROJECT_ROOT/target/universal-apple-darwin/release"
MACOS_APP_DIR="$MACOS_RELEASE_DIR/Browsers.app"

# Load in some secrets
source "$PROJECT_ROOT/.env"

DMG_FROM_DIR="dmg_source"

# Will also be used as the title of the installer
DMG_SRC_DIR="Browsers"

echo "Creating dmg"
mkdir -p $DMG_SRC_DIR
mkdir $DMG_SRC_DIR/.background

cp -r $MACOS_APP_DIR $DMG_SRC_DIR/Browsers.app
cp $DMG_FROM_DIR/.VolumeIcon.icns $DMG_SRC_DIR/.VolumeIcon.icns

# Copies Finder settings (like custom background color and icon placements)
cp $DMG_FROM_DIR/.DS_Store $DMG_SRC_DIR/.DS_Store

ln -s /Applications $DMG_SRC_DIR/Applications
rm -rf $DMG_SRC_DIR/.Trashes

hdiutil create -volname "Browsers" -srcfolder $DMG_SRC_DIR -ov "$MACOS_RELEASE_DIR/Browsers.dmg"

# set creator of the file
#SetFile -c icnC "$DMG_FROM_DIR/.VolumeIcon.icns"

cp $DMG_FROM_DIR/.VolumeIcon.icns "$DMG_SRC_DIR/copy_VolumeIcon.icns"
sips -i "$DMG_SRC_DIR/copy_VolumeIcon.icns"

# let's reuse the temp DMG_SRC_DIR for temp files
DeRez -only icns "$DMG_SRC_DIR/copy_VolumeIcon.icns" > "$DMG_SRC_DIR/copy_VolumeIcon.rsrc"
Rez -append "$DMG_SRC_DIR/copy_VolumeIcon.rsrc" -o "$MACOS_RELEASE_DIR/Browsers.dmg"

# Let's tell the file it has special icon flag set
SetFile -a C "$MACOS_RELEASE_DIR/Browsers.dmg"

rm -rf $DMG_SRC_DIR


codesign -s "$APP_CERT_ID" -f -o runtime --timestamp "$MACOS_RELEASE_DIR/Browsers.dmg"

# Once signed you can do the notarization
# https://gregoryszorc.com/docs/apple-codesign/main/apple_codesign_getting_started.html#apple-codesign-app-store-connect-api-key

rcodesign notary-submit \
  --api-key-path ~/.appstoreconnect/key.json \
  --staple \
  "$MACOS_RELEASE_DIR/Browsers.dmg"
