use errors::*;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

/// Takes a path to a file and try to read the file into a String
pub fn file_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();

    let mut content = String::new();
    File::open(path)
        .chain_err(|| "Unable to open the file")?
        .read_to_string(&mut content)
        .chain_err(|| "Unable to read the file")?;

    Ok(content)
}

/// Naively replaces any path seperator with a forward-slash '/'
pub fn normalize_path(path: &str) -> String {
    use std::path::is_separator;
    path.chars()
        .map(|ch| if is_separator(ch) { '/' } else { ch })
        .collect::<String>()
}

/// Write the given data to a file, creating it first if necessary
pub fn write_file<P: AsRef<Path>>(
    build_dir: &Path,
    filename: P,
    content: &[u8],
) -> Result<()> {
    let path = build_dir.join(filename);

    create_file(&path)?
        .write_all(content)
        .map_err(|e| e.into())
}

/// Takes a path and returns a path containing just enough `../` to point to
/// the root of the given path.
///
/// This is mostly interesting for a relative path to point back to the
/// directory from where the path starts.
///
/// ```rust
/// # extern crate mdbook;
/// #
/// # use std::path::Path;
/// # use mdbook::utils::fs::path_to_root;
/// #
/// # fn main() {
/// let path = Path::new("some/relative/path");
/// assert_eq!(path_to_root(path), "../../");
/// # }
/// ```
///
/// **note:** it's not very fool-proof, if you find a situation where
/// it doesn't return the correct path.
/// Consider [submitting a new issue](https://github.com/rust-lang-nursery/mdBook/issues)
/// or a [pull-request](https://github.com/rust-lang-nursery/mdBook/pulls) to improve it.
pub fn path_to_root<P: Into<PathBuf>>(path: P) -> String {
    debug!("path_to_root");
    // Remove filename and add "../" for every directory

    path.into()
        .parent()
        .expect("")
        .components()
        .fold(String::new(), |mut s, c| {
            match c {
                Component::Normal(_) => s.push_str("../"),
                _ => {
                    debug!("Other path component... {:?}", c);
                }
            }
            s
        })
}

/// This function creates a file and returns it. But before creating the file
/// it checks every directory in the path to see if it exists,
/// and if it does not it will be created.
pub fn create_file(path: &Path) -> Result<File> {
    debug!("Creating {}", path.display());

    // Construct path
    if let Some(p) = path.parent() {
        trace!("Parent directory is: {:?}", p);

        fs::create_dir_all(p)?;
    }

    File::create(path).map_err(|e| e.into())
}

/// Removes all the content of a directory but not the directory itself
pub fn remove_dir_content(dir: &Path) -> Result<()> {
    for item in fs::read_dir(dir)? {
        if let Ok(item) = item {
            let item = item.path();
            if item.is_dir() {
                fs::remove_dir_all(item)?;
            } else {
                fs::remove_file(item)?;
            }
        }
    }
    Ok(())
}

/// Copies all files of a directory to another one except the files
/// with the extensions given in the `ext_blacklist` array
pub fn copy_files_except_ext(
    from: &Path,
    to: &Path,
    recursive: bool,
    ext_blacklist: &[&str],
) -> Result<()> {
    debug!(
        "Copying all files from {} to {} (blacklist: {:?})",
        from.display(),
        to.display(),
        ext_blacklist
    );

    // Check that from and to are different
    if from == to {
        return Ok(());
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        // If the entry is a dir and the recursive option is enabled, call itself
        if metadata.is_dir() && recursive {
            if entry.path() == to.to_path_buf() {
                continue;
            }

            // check if output dir already exists
            if !to.join(entry.file_name()).exists() {
                fs::create_dir(&to.join(entry.file_name()))?;
            }

            copy_files_except_ext(
                &from.join(entry.file_name()),
                &to.join(entry.file_name()),
                true,
                ext_blacklist,
            )?;
        } else if metadata.is_file() {
            // Check if it is in the blacklist
            if let Some(ext) = entry.path().extension() {
                if ext_blacklist.contains(&ext.to_str().unwrap()) {
                    continue;
                }
            }
            debug!(
                "creating path for file: {:?}",
                &to.join(
                    entry
                        .path()
                        .file_name()
                        .expect("a file should have a file name...")
                )
            );

            debug!(
                "Copying {:?} to {:?}",
                entry.path(),
                &to.join(
                    entry
                        .path()
                        .file_name()
                        .expect("a file should have a file name...")
                )
            );
            fs::copy(
                entry.path(),
                &to.join(
                    entry
                        .path()
                        .file_name()
                        .expect("a file should have a file name..."),
                ),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate tempfile;

    use super::copy_files_except_ext;
    use std::fs;

    #[test]
    fn copy_files_except_ext_test() {
        let tmp = match tempfile::TempDir::new() {
            Ok(t) => t,
            Err(_) => panic!("Could not create a temp dir"),
        };

        // Create a couple of files
        if let Err(_) = fs::File::create(&tmp.path().join("file.txt")) {
            panic!("Could not create file.txt")
        }
        if let Err(_) = fs::File::create(&tmp.path().join("file.md")) {
            panic!("Could not create file.md")
        }
        if let Err(_) = fs::File::create(&tmp.path().join("file.png")) {
            panic!("Could not create file.png")
        }
        if let Err(_) = fs::create_dir(&tmp.path().join("sub_dir")) {
            panic!("Could not create sub_dir")
        }
        if let Err(_) = fs::File::create(&tmp.path().join("sub_dir/file.png")) {
            panic!("Could not create sub_dir/file.png")
        }
        if let Err(_) = fs::create_dir(&tmp.path().join("sub_dir_exists")) {
            panic!("Could not create sub_dir_exists")
        }
        if let Err(_) = fs::File::create(&tmp.path().join("sub_dir_exists/file.txt")) {
            panic!("Could not create sub_dir_exists/file.txt")
        }

        // Create output dir
        if let Err(_) = fs::create_dir(&tmp.path().join("output")) {
            panic!("Could not create output")
        }
        if let Err(_) = fs::create_dir(&tmp.path().join("output/sub_dir_exists")) {
            panic!("Could not create output/sub_dir_exists")
        }

        match copy_files_except_ext(&tmp.path(), &tmp.path().join("output"), true, &["md"]) {
            Err(e) => panic!("Error while executing the function:\n{:?}", e),
            Ok(_) => {}
        }

        // Check if the correct files where created
        if !(&tmp.path().join("output/file.txt")).exists() {
            panic!("output/file.txt should exist")
        }
        if (&tmp.path().join("output/file.md")).exists() {
            panic!("output/file.md should not exist")
        }
        if !(&tmp.path().join("output/file.png")).exists() {
            panic!("output/file.png should exist")
        }
        if !(&tmp.path().join("output/sub_dir/file.png")).exists() {
            panic!("output/sub_dir/file.png should exist")
        }
        if !(&tmp.path().join("output/sub_dir_exists/file.txt")).exists() {
            panic!("output/sub_dir/file.png should exist")
        }
    }
}
