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

pub fn find_spool(name: &str, version: &str) -> io::Result<Option<SpoolEntry>> {
    let root = fetch_registry()?;
    let path = spool_path(&root, name, version);
    if !path.exists() {
        return Ok(None);
    }
    SpoolEntry::read(&path).map(Some)
}

pub fn search_by_name(name: &str) -> io::Result<Vec<SpoolEntry>> {
    let entries = list_spools()?;
    let needle = name.to_lowercase();
    Ok(entries
        .into_iter()
        .filter(|entry| entry.name.to_lowercase().contains(&needle))
        .collect())
}

pub fn spool_path(root: &Path, name: &str, version: &str) -> PathBuf {
    root.join("entries")
        .join(bucket_for_name(name))
        .join(name)
        .join(version)
        .join("spool.toml")
}

pub fn bucket_for_name(name: &str) -> &'static str {
    let lower = name.trim().to_ascii_lowercase();
    if lower.starts_with("lib") {
        return "lib";
    }

    match lower.bytes().next() {
        Some(first @ b'a'..=b'z') => match first {
            b'a' => "a",
            b'b' => "b",
            b'c' => "c",
            b'd' => "d",
            b'e' => "e",
            b'f' => "f",
            b'g' => "g",
            b'h' => "h",
            b'i' => "i",
            b'j' => "j",
            b'k' => "k",
            b'l' => "l",
            b'm' => "m",
            b'n' => "n",
            b'o' => "o",
            b'p' => "p",
            b'q' => "q",
            b'r' => "r",
            b's' => "s",
            b't' => "t",
            b'u' => "u",
            b'v' => "v",
            b'w' => "w",
            b'x' => "x",
            b'y' => "y",
            b'z' => "z",
            _ => "lib",
        },
        _ => "lib",
    }
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
