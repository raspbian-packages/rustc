#!/usr/bin/python
import json
import sys

for i in sys.argv[1:]:
	with open(i, "r+") as fp:
		x = json.load(fp)
		x["files"] = {}
		fp.seek(0)
		json.dump(x, fp, separators=(',', ':'))
		fp.truncate()
