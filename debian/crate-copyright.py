#!/bin/python3

import datetime
import pytoml
import os
import subprocess
import sys

this_year = datetime.datetime.now().year
crates = sys.argv[1:]
get_initial_commit = len(crates) == 1

for crate in crates:
	with open(os.path.join(crate, "Cargo.toml")) as fp:
		data = pytoml.load(fp)
		repo = data["package"].get("repository", None)
		if get_initial_commit and repo:
			output = subprocess.check_output(
				"""git clone --bare "%s" tmp.crate-copyright >&2 &&
cd tmp.crate-copyright &&
git log --format=%%cI --reverse | head -n1 | cut -b1-4 &&
git log --format=%%cI           | head -n1 | cut -b1-4 &&
cd .. &&
rm -rf tmp.crate-copyright""" % repo, shell=True).decode("utf-8")
			first_year, last_year = output.strip().split(maxsplit=2)
		else:
			first_year = "20XX"
			last_year = this_year
		print("""Files: {0}
Copyright: {1}
License: {2}
Comment: see {3}
""".format(
		os.path.join(crate, "*"),
		"\n           ".join("%s-%s %s" % (first_year, last_year, a.replace(" <>", "")) for a in data ["package"]["authors"]),
		data["package"].get("license", "???").replace("/", " or ").replace("MIT", "Expat"),
		repo or "???"
	))
