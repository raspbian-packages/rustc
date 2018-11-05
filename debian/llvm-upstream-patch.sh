#!/bin/sh
# Run this on https://github.com/llvm-mirror/llvm
# Or another repo where the above is the "upstream" remote
set -e
head=$(git rev-parse --verify -q remotes/upstream/master || git rev-parse --verify -q remotes/origin/master)
test -n "$head"
for i in "$@"; do
	git show $(git rev-list "$head" -n1 --grep='git-svn-id: .*@'"$i") > rL"$i".patch
done
