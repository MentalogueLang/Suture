use std::io;
use std::path::PathBuf;

use crate::cache::spools_root;

pub fn fetch_registry() -> io::Result<PathBuf> {
    let root = spools_root()?;
    if !root.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("spools registry not found at `{}`", root.display()),
        ));
    }
    Ok(root)
}
