// error-pattern: error reading Clippy's configuration file: unknown key `foobar`

#![feature(plugin)]
#![plugin(clippy(conf_file="./tests/auxiliary/conf_unknown_key.toml"))]

fn main() {}
