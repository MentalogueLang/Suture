use std::io;
use std::path::PathBuf;

pub fn spools_root() -> io::Result<PathBuf> {
    if let Ok(value) = std::env::var("SUTURE_SPOOLS_DIR") {
        if !value.is_empty() {
            return Ok(PathBuf::from(value));
        }
    }

    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing home directory"))?;

    Ok(home.join(".mentalogue").join("suture").join("spools-index"))
}
