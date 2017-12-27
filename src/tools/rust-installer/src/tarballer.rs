// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::thread;

use flate2;
use flate2::write::GzEncoder;
use tar::{Builder, Header};
use walkdir::WalkDir;
use xz2::write::XzEncoder;

use errors::*;
use util::*;

actor!{
    #[derive(Debug)]
    pub struct Tarballer {
        /// The input folder to be compressed
        input: String = "package",

        /// The prefix of the tarballs
        output: String = "./dist",

        /// The folder in which the input is to be found
        work_dir: String = "./workdir",
    }
}

impl Tarballer {
    /// Generate the actual tarballs
    pub fn run(self) -> Result<()> {
        let tar_gz = self.output.clone() + ".tar.gz";
        let tar_xz = self.output.clone() + ".tar.xz";

        // Remove any existing files
        for file in &[&tar_gz, &tar_xz] {
            if Path::new(file).exists() {
                remove_file(file)?;
            }
        }

        // Sort files by their suffix, to group files with the same name from
        // different locations (likely identical) and files with the same
        // extension (likely containing similar data).
        let (dirs, mut files) = get_recursive_paths(&self.work_dir, &self.input)
            .chain_err(|| "failed to collect file paths")?;
        files.sort_by(|a, b| a.bytes().rev().cmp(b.bytes().rev()));

        // Write the tar into both encoded files.  We write all directories
        // first, so files may be directly created. (see rustup.rs#1092)
        let mut builder = Builder::new(Vec::new());
        for path in dirs {
            let src = Path::new(&self.work_dir).join(&path);
            builder.append_dir(&path, &src)
                .chain_err(|| format!("failed to tar dir '{}'", src.display()))?;
        }
        for path in files {
            let src = Path::new(&self.work_dir).join(&path);
            let file = open_file(&src)?;
            builder.append_data(&mut header(&src, &file)?, &path, &file)
                .chain_err(|| format!("failed to tar file '{}'", src.display()))?;
        }
        let contents = builder.into_inner()
            .chain_err(|| "failed to finish writing .tar stream")?;
        let contents = Arc::new(contents);

        // Prepare the .tar.gz file
        let contents2 = contents.clone();
        let t = thread::spawn(move || {
            let mut gz = GzEncoder::new(create_new_file(tar_gz)?,
                                        flate2::Compression::Best);
            gz.write_all(&contents2).chain_err(|| "failed to write .gz")?;
            gz.finish().chain_err(|| "failed to finish .gz")
        });

        // Prepare the .tar.xz file
        let mut xz = XzEncoder::new(create_new_file(tar_xz)?, 9);
        xz.write_all(&contents).chain_err(|| "failed to write .xz")?;
        xz.finish().chain_err(|| "failed to finish .xz")?;

        t.join().unwrap()?;

        Ok(())
    }
}

fn header(src: &Path, file: &File) -> Result<Header> {
    let mut header = Header::new_gnu();
    header.set_metadata(&file.metadata()?);
    if cfg!(windows) {
        // Windows doesn't really have a mode, so `tar` never marks files executable.
        // Use an extension whitelist to update files that usually should be so.
        const EXECUTABLES: [&'static str; 4] = ["exe", "dll", "py", "sh"];
        if let Some(ext) = src.extension().and_then(|s| s.to_str()) {
            if EXECUTABLES.contains(&ext) {
                let mode = header.mode()?;
                header.set_mode(mode | 0o111);
            }
        }
    }
    Ok(header)
}

/// Returns all `(directories, files)` under the source path
fn get_recursive_paths<P, Q>(root: P, name: Q) -> Result<(Vec<String>, Vec<String>)>
    where P: AsRef<Path>, Q: AsRef<Path>
{
    let root = root.as_ref();
    let name = name.as_ref();

    if !name.is_relative() && !name.starts_with(root) {
        bail!("input '{}' is not in work dir '{}'", name.display(), root.display());
    }

    let mut dirs = vec![];
    let mut files = vec![];
    for entry in WalkDir::new(root.join(name)) {
        let entry = entry?;
        let path = entry.path().strip_prefix(root)?;
        let path = path_to_str(&path)?;

        if entry.file_type().is_dir() {
            dirs.push(path.to_owned());
        } else {
            files.push(path.to_owned());
        }
    }
    Ok((dirs, files))
}
