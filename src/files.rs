use super::*;
use std::path::Path;

/// Recursively walk through a directory, calling callback
pub fn walk_recursively<'a>(
    generator: &mut OreStaty<'a>,
    dir: &Path,
    path: &Path,
    callback: &mut impl FnMut(&mut OreStaty<'a>, &Path, &Path),
) {
    for file in unwrap_or_error!(
        std::fs::read_dir(dir.join(path)),
        generator=>err("Failed to read directory {:?}: {}", dir, err),
        return
    ) {
        let file = unwrap_or_error!(
            file,
            generator=>err("Failed to read file: {}", err),
            continue
        );
        let name = std::path::PathBuf::from(file.file_name());
        let path = path.join(&name);

        if file.path().is_file() {
            callback(generator, &file.path(), &path);
        } else {
            walk_recursively(generator, dir, &path, callback);
        }
    }
}

/// Copy all files from src to dst recursively
pub fn copy_recursively(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    // Source: https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let copy = || -> std::io::Result<()> {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                copy_recursively(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
            Ok(())
        };
        if let Err(err) = copy() {
            eprintln!("Failed to copy asset: {}", err);
            continue;
        };
    }
    Ok(())
}
