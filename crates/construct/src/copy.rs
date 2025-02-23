use std::path::{Path, PathBuf};

use loss72_platemaker_core::{fs::{Directory, File}, log};

pub fn copy_dir_recursively(
    dir: &Directory,
    dest: &Directory,
    excludes: &[PathBuf],
) -> Result<(), std::io::Error> {
    copy_files(
        dir,
        dest,
        &dir.try_iter_tree()?
            .into_iter()
            .filter(|file| {
                let Ok(file) = file else { return true; };
                !excluded(dir.path(), file.path(), &excludes)
            })
            .collect::<Result<Vec<_>, _>>()?
    )
}

pub fn copy_files(
    dir: &Directory,
    dest: &Directory,
    files: &[File]
) -> Result<(), std::io::Error> {
    for file in files {
        log!(step: "Copying file: {}", file.path().display());

        let subpath_in_dest = file.path().strip_prefix(dir.path()).unwrap_or(file.path());
        let dest = dest.path().join(subpath_in_dest);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(file.path(), dest)?;
    }

    log!(ok: "Copied");

    Ok(())
}

fn excluded(root: &Path, path: &Path, excludes: &[PathBuf]) -> bool {
    excludes.iter().any(|excluding| {
        if excluding.is_absolute() {
            path == excluding
        } else if excluding.is_relative() {
            path == root.join(excluding)
        } else {
            false
        }
    })
}

