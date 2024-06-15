use std::path::Path;

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
