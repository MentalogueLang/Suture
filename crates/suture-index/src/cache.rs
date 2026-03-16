use std::io;
use std::path::PathBuf;

pub fn spools_root() -> io::Result<PathBuf> {
    if let Ok(value) = std::env::var("SUTURE_SPOOLS_DIR") {
        if !value.is_empty() {
            return Ok(PathBuf::from(value));
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing workspace root"))?;

    Ok(repo_root.join("spools"))
}
