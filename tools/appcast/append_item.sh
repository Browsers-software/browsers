#!/usr/bin/env bash

# exit when any command fails
set -e

# append item to appcast
append() {
  # create item.xml for new version
  ./create_item.sh > item.xml

  # add 8 spaces
  sed  's/^/        /'  item.xml > item_indented.xml

  # insert item_indented.xml after line 7 in appcast.xml
  sed '7 r item_indented.xml' appcast.xml > appcast_new.xml
}

append