use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::cache::spools_root;

pub fn fetch_registry() -> io::Result<PathBuf> {
    let root = spools_root()?;
    if root.join(".git").exists() {
        git(["-C", path_arg(&root)?, "pull", "--ff-only"])?;
        return Ok(root);
    }

    if root.exists() {
        return Ok(root);
    }

    if let Some(parent) = root.parent() {
        fs::create_dir_all(parent)?;
    }
    let remote = std::env::var("SUTURE_SPOOLS_REPO_URL")
        .unwrap_or_else(|_| "https://github.com/MentalogueLang/Spools-Index.git".to_string());
    git(["clone", remote.as_str(), path_arg(&root)?])?;
    Ok(root)
}

fn git<const N: usize>(args: [&str; N]) -> io::Result<()> {
    let status = Command::new("git").args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("git failed with status {status}"),
        ))
    }
}

fn path_arg(path: &PathBuf) -> io::Result<&str> {
    path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path `{}` is not valid UTF-8", path.display()),
        )
    })
}
