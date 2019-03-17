#!/bin/sh
# Pipe the output of lintian into this.
sed -ne 's/.* file-without-copyright-information //p' | cut -d/ -f1-2 | sort -u | while read x; do
	/usr/share/cargo/scripts/guess-crate-copyright "$x"
done
