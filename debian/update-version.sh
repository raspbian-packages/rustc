#!/bin/bash
# Don't run this directly, use "debian/rules update-version" instead

prev_stable() {
local V=$1
python -c 'import sys; k=map(int,sys.argv[1].split(".")); k[1]-=1; print ".".join(map(str,k))' "$V"
}

update() {
local ORIG=$1 NEW=$2

ORIG_M1=$(prev_stable $ORIG)
NEW_M1=$(prev_stable $NEW)
ORIG_R="${ORIG/./\\.}" # match a literal dot, otherwise this might sometimes match e.g. debhelper (>= 9.20141010)

sed -i -e "s|libstd-rust-${ORIG_R}|libstd-rust-$NEW|g" \
       -e "s|rustc\( *\)(<= ${ORIG_R}|rustc\1(<= $NEW|g" \
       -e "s|rustc\( *\)(>= ${ORIG_M1/./\\.}|rustc\1(>= ${NEW_M1}|g" control

git mv libstd-rust-$ORIG.lintian-overrides libstd-rust-$NEW.lintian-overrides
sed -i -e "s|libstd-rust-${ORIG_R}|libstd-rust-$NEW|g" libstd-rust-$NEW.lintian-overrides
}

cd $(dirname "$0")
update "$1" "$2"
