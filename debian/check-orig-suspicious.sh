#!/bin/bash
set -e

ver="$1"
test -n "$ver" || exit 2

SUS_WHITELIST=$(find "${PWD}/debian" -name upstream-tarball-unsuspicious.txt -type f)

rm -rf rustc-${ver/*~*/beta}-src/
tar xf ../rustc_$ver+dfsg1.orig.tar.xz && cd rustc-${ver/*~*/beta}-src/

/usr/share/cargo/scripts/audit-vendor-source \
  "$SUS_WHITELIST" \
  "Files-Excluded: in debian/copyright and run a repack."

echo "Artifacts left in rustc-$ver-src, please remove them yourself."
