use std::io;
use std::path::{Path, PathBuf};

use crate::fetch::fetch_registry;
use crate::spool_entry::SpoolEntry;

pub fn list_spools() -> io::Result<Vec<SpoolEntry>> {
    let root = fetch_registry()?;
    let mut files = Vec::new();
    collect_spool_files(&root, &mut files)?;
    let mut entries = Vec::new();
    for file in files {
        if let Ok(entry) = SpoolEntry::read(&file) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

pub fn search_by_name(name: &str) -> io::Result<Vec<SpoolEntry>> {
    let entries = list_spools()?;
    let needle = name.to_lowercase();
    Ok(entries
        .into_iter()
        .filter(|entry| entry.name.to_lowercase().contains(&needle))
        .collect())
}

fn collect_spool_files(root: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_spool_files(&path, files)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("spool.toml"))
        {
            files.push(path);
        }
    }
    Ok(())
}
