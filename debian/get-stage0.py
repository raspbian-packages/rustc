#!/usr/bin/python
# Sometimes this might fail due to upstream changes.
# In that case, you probably just need to override the failing step in our
# DownloadOnlyRustBuild class below.

import sys

import bootstrap
from bootstrap import RustBuild

class DownloadOnlyRustBuild(RustBuild):
    triple = None
    def build_bootstrap(self):
        pass
    def run(self, *args):
        pass
    def build_triple(self):
        return self.triple
    def update_submodules(self):
        pass
    def bootstrap_binary(self):
        return "true"

def main(argv):
    triple = argv.pop(1)
    DownloadOnlyRustBuild.triple = triple
    bootstrap.RustBuild = DownloadOnlyRustBuild
    bootstrap.bootstrap()

if __name__ == '__main__':
    main(sys.argv)
