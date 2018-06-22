#!/bin/sh
# See README.Debian "Bootstrapping" for details.
#
# You may want to use `debian/rules source_orig-stage0` instead of calling this
# directly.

set -e

upstream_version="$(dpkg-parsechangelog -SVersion | sed -e 's/\(.*\)-.*/\1/g')"
upstream_bootstrap_arch="${upstream_bootstrap_arch:-amd64 arm64 armhf i386 mips64 mips64el powerpc ppc64 ppc64el s390x}"

rm -f stage0/*/*.sha256
mkdir -p stage0 build && ln -sf ../stage0 build/cache
if [ -n "$(find stage0/ -type f)" ]; then
	echo >&2 "$0: NOTE: extra artifacts in stage0/ will be included:"
	find stage0/ -type f
fi
for deb_host_arch in $upstream_bootstrap_arch; do
	make -s --no-print-directory -f debian/architecture-test.mk "rust-for-deb_${deb_host_arch}" | {
	read deb_host_arch rust_triplet
	PYTHONPATH=src/bootstrap debian/get-stage0.py "${rust_triplet}"
	rm -rf "${rust_triplet}"
	}
done

echo >&2 "building stage0 tar file now, this will take a while..."
stamp=@${SOURCE_DATE_EPOCH:-$(date +%s)}
touch --date="$stamp" stage0/dpkg-source-dont-rename-parent-directory
tar --mtime="$stamp" --clamp-mtime \
  --owner=root --group=root \
  -cJf "../rustc_${upstream_version}.orig-stage0.tar.xz" \
  --transform "s/^stage0\///" \
  stage0/*

rm -f src/bootstrap/bootstrap.pyc

cat <<eof
================================================================================
orig-stage0 bootstrapping tarball created in ../rustc_${upstream_version}.orig-stage0.tar.xz
containing the upstream compilers for $upstream_bootstrap_arch

You *probably* now want to do the following steps:

1. Add [$(echo $upstream_bootstrap_arch | sed -e 's/\S*/!\0/g')] to the rustc/cargo Build-Depends in d/control
2. Update d/changelog
3. Run \`dpkg-source -b .\` to generate a .dsc that includes this tarball.
================================================================================
eof
