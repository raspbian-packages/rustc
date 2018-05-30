// error-pattern:yummy
#![feature(box_syntax)]
#![feature(rustc_private)]
#![allow(unknown_lints, missing_docs_in_private_items)]

const CARGO_CLIPPY_HELP: &str = r#"Checks a package to catch common mistakes and improve your Rust code.

Usage:
    cargo clippy [options] [--] [<opts>...]

Common options:
    -h, --help               Print this message
    -V, --version            Print version info and exit

Other options are the same as `cargo check`.

To allow or deny a lint from the command line you can use `cargo clippy --`
with:

    -W --warn OPT       Set lint warnings
    -A --allow OPT      Set lint allowed
    -D --deny OPT       Set lint denied
    -F --forbid OPT     Set lint forbidden

The feature `cargo-clippy` is automatically defined for convenience. You can use
it to allow or deny lints from the code, eg.:

    #[cfg_attr(feature = "cargo-clippy", allow(needless_lifetimes))]
"#;

#[allow(print_stdout)]
fn show_help() {
    println!("{}", CARGO_CLIPPY_HELP);
}

#[allow(print_stdout)]
fn show_version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}

pub fn main() {
    // Check for version and help flags even when invoked as 'cargo-clippy'
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        show_help();
        return;
    }
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        show_version();
        return;
    }

    if let Err(code) = process(std::env::args().skip(2)) {
        std::process::exit(code);
    }
}

fn process<I>(mut old_args: I) -> Result<(), i32>
where
    I: Iterator<Item = String>,
{
    let mut args = vec!["check".to_owned()];

    let mut found_dashes = false;
    for arg in old_args.by_ref() {
        found_dashes |= arg == "--";
        if found_dashes {
            break;
        }
        args.push(arg);
    }

    let clippy_args: String = old_args.map(|arg| format!("{}__CLIPPY_HACKERY__", arg)).collect();

    let mut path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name("clippy-driver");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    let exit_status = std::process::Command::new("cargo")
        .args(&args)
        .env("RUSTC_WRAPPER", path)
        .env("CLIPPY_ARGS", clippy_args)
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if exit_status.success() {
        Ok(())
    } else {
        Err(exit_status.code().unwrap_or(-1))
    }
}
