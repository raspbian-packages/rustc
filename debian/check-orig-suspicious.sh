#!/bin/bash

set -ex

ver="$1"
test -n "$ver" || exit 2

FILTER="Files-Excluded: in debian/copyright and run a repack."
SUS_WHITELIST=$(find "${PWD}" -name upstream-tarball-unsuspicious.txt -type f)

rm -rf rustc-${ver/*~*/beta}-src/
tar xf ../rustc_$ver+dfsg1.orig.tar.xz && cd rustc-${ver/*~*/beta}-src/

# Remove tiny files 4 bytes or less
find . -size -4c -delete
# Remove non-suspicious files, warning on patterns that match nothing
grep -v '^#' ${SUS_WHITELIST} | xargs  -I% sh -c 'rm -r ./% || true'
echo "Checking for suspicious files..."

# TODO: merge the -m stuff into suspicious-source(1).
suspicious-source -v -m text/x-objective-c
# The following shell snippet is a bit more strict than suspicious-source(1)
find . -type f -and -not -name '.cargo-checksum.json' -exec file '{}' \; | \
  sed -e 's/\btext\b\(.*\), with very long lines/verylongtext\1/g' | \
  grep -v '\b\(text\|empty\)\b' || true

# Most C and JS code should be in their own package
find src/vendor/ -name '*.c' -o -name '*.js'

echo "The above files (if any) seem suspicious, please audit them."
echo "If good, add them to ${SUS_WHITELIST}."
echo "If bad, add them to ${FILTER}."

echo "Artifacts left in rustc-$ver-src, please remove them yourself."
