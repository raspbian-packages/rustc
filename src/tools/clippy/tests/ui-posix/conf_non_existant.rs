// error-pattern: error reading Clippy's configuration file

#![feature(plugin)]
#![plugin(clippy(conf_file="./tests/auxiliary/non_existant_conf.toml"))]

fn main() {}
