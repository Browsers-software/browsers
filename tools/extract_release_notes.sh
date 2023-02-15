#!/usr/bin/env bash

# Based on https://stackoverflow.com/a/68119286

awk -v ver="$1" '
 /^#+ \[/ { if (p) { exit }; if ($2 == "["ver"]") { p=1; next} } p && NF
' "$2"