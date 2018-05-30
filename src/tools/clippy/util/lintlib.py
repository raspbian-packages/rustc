# Common utils for the several housekeeping scripts.

import os
import re
import collections

import logging as log
log.basicConfig(level=log.INFO, format='%(levelname)s: %(message)s')

Lint = collections.namedtuple('Lint', 'name level doc sourcefile')
Config = collections.namedtuple('Config', 'name ty doc default')

lintname_re = re.compile(r'''pub\s+([A-Z_][A-Z_0-9]*)''')
level_re = re.compile(r'''(Forbid|Deny|Warn|Allow)''')
group_re = re.compile(r'''([a-z_][a-z_0-9]+)''')
conf_re = re.compile(r'''define_Conf! {\n([^}]*)\n}''', re.MULTILINE)
confvar_re = re.compile(
    r'''/// Lint: (\w+). (.*).*\n\s*\([^,]+,\s+"([^"]+)",\s+([^=\)]+)=>\s+(.*)\),''', re.MULTILINE)

lint_levels = {
    "correctness": 'Deny',
    "style": 'Warn',
    "complexity": 'Warn',
    "perf": 'Warn',
    "restriction": 'Allow',
    "pedantic": 'Allow',
    "nursery": 'Allow',
}

def parse_lints(lints, filepath):
    last_comment = []
    comment = True
    clippy = False
    deprecated = False
    name = ""

    with open(filepath) as fp:
        for line in fp:
            if comment:
                if line.startswith("/// "):
                    last_comment.append(line[4:])
                elif line.startswith("///"):
                    last_comment.append(line[3:])
                elif line.startswith("declare_lint!"):
                    import sys
                    print "don't use `declare_lint!` in clippy, use `declare_clippy_lint!` instead"
                    sys.exit(42)
                elif line.startswith("declare_clippy_lint!"):
                    comment = False
                    deprecated = False
                    clippy = True
                    name = ""
                elif line.startswith("declare_deprecated_lint!"):
                    comment = False
                    deprecated = True
                    clippy = False
                else:
                    last_comment = []
            if not comment:
                if name:
                    g = group_re.search(line)
                    if g:
                        group = g.group(1).lower()
                        level = lint_levels[group]
                        log.info("found %s with level %s in %s",
                                name, level, filepath)
                        lints.append(Lint(name, level, last_comment, filepath, group))
                        last_comment = []
                        comment = True
                else:
                    m = lintname_re.search(line)
                    if m:
                        name = m.group(1).lower()

                        if deprecated:
                            level = "Deprecated"
                        else:
                            while True:
                                m = level_re.search(line)
                                if m:
                                    level = m.group(0)
                                    break
                                line = next(fp)
                        if not clippy:
                            log.info("found %s with level %s in %s",
                                    name, level, filepath)
                            lints.append(Lint(name, level, last_comment, filepath, "deprecated"))
                            last_comment = []
                            comment = True
                    if "}" in line:
                        log.warn("Warning: missing Lint-Name in %s", filepath)
                        comment = True


def parse_configs(path):
    configs = {}
    with open(os.path.join(path, 'utils/conf.rs')) as fp:
        contents = fp.read()

    match = re.search(conf_re, contents)
    confvars = re.findall(confvar_re, match.group(1))

    for (lint, doc, name, default, ty) in confvars:
        configs[lint.lower()] = Config(name.replace("_", "-"), ty, doc, default)

    return configs


def parse_all(path="clippy_lints/src"):
    lints = []
    for filename in os.listdir(path):
        if filename.endswith(".rs"):
            parse_lints(lints, os.path.join(path, filename))
    log.info("got %s lints", len(lints))

    configs = parse_configs(path)
    log.info("got %d configs", len(configs))

    return lints, configs
