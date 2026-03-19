use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use suture_index::{find_spool, SpoolEntry};
use suture_manifest::ProjectManifest;

pub fn project_dir() -> Result<PathBuf, String> {
    std::env::current_dir().map_err(|error| error.to_string())
}

pub fn manifest_path(dir: &Path) -> PathBuf {
    ProjectManifest::path_for_dir(dir)
}

pub fn load_manifest(dir: &Path) -> Result<ProjectManifest, String> {
    ProjectManifest::read_or_default(&manifest_path(dir)).map_err(|error| error.to_string())
}

pub fn save_manifest(dir: &Path, manifest: &ProjectManifest) -> Result<PathBuf, String> {
    let path = manifest_path(dir);
    manifest.write(&path).map_err(|error| error.to_string())?;
    Ok(path)
}

pub fn suture_root(dir: &Path) -> PathBuf {
    dir.join(".suture")
}

pub fn source_checkout_dir(dir: &Path, name: &str, version: &str) -> PathBuf {
    suture_root(dir).join("sources").join(name).join(version)
}

pub fn built_mlib_path(dir: &Path, name: &str, version: &str) -> PathBuf {
    suture_root(dir)
        .join("mlib")
        .join(name)
        .join(version)
        .join(format!("{name}.mlib"))
}

pub fn find_exact_spool(name: &str, version: &str) -> Result<SpoolEntry, String> {
    find_spool(name, version)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("spool `{name}` version `{version}` not found in the index"))
}

pub fn sync_spool_source(entry: &SpoolEntry, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    if destination.join(".git").exists() {
        run_git(
            ["-C", as_arg(destination)?, "fetch", "--all", "--tags", "--force"],
            None,
        )?;
        run_git(["-C", as_arg(destination)?, "reset", "--hard", "HEAD"], None)?;
    } else {
        if destination.exists() {
            fs::remove_dir_all(destination).map_err(|error| error.to_string())?;
        }
        run_git(
            ["clone", entry.source_git.as_str(), as_arg(destination)?],
            None,
        )?;
    }

    if let Some(rev) = entry.source_rev.as_deref() {
        run_git(["-C", as_arg(destination)?, "checkout", "--force", rev], None)?;
    } else if let Some(tag) = entry.source_tag.as_deref() {
        run_git(
            ["-C", as_arg(destination)?, "checkout", "--force", tag],
            None,
        )?;
    }

    Ok(())
}

pub fn resolve_build_entry(entry: &SpoolEntry, source_root: &Path) -> Result<PathBuf, String> {
    let candidates = [
        entry.build_entry.as_deref(),
        Some("lib.mtl"),
        Some("src/lib.mtl"),
        Some("main.mtl"),
        Some("src/main.mtl"),
    ];

    for candidate in candidates.into_iter().flatten() {
        let path = source_root.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(format!(
        "unable to find build entry for `{}`; set `[build] entry = \"...\"` in spool.toml",
        entry.name
    ))
}

pub fn emit_mlib(entry_source: &Path, output: &Path) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let target = host_target();
    let input = entry_source.to_string_lossy().to_string();
    let output_arg = output.to_string_lossy().to_string();
    let args = [
        "emit",
        "mlib",
        input.as_str(),
        "--target",
        target,
        "-o",
        output_arg.as_str(),
    ];

    let mut failures = Vec::new();

    if let Ok(value) = std::env::var("SUTURE_INSCRIBE_BIN") {
        if !value.trim().is_empty() {
            match run_command(Command::new(value).args(args), "inscribe emit mlib") {
                Ok(()) => return Ok(()),
                Err(error) => failures.push(error),
            }
        }
    }

    match run_command(Command::new("inscribe").args(args), "inscribe emit mlib") {
        Ok(()) => return Ok(()),
        Err(error) => failures.push(error),
    }

    match run_command(
        Command::new("inscribe-cli").args(args),
        "inscribe-cli emit mlib",
    ) {
        Ok(()) => return Ok(()),
        Err(error) => failures.push(error),
    }

    if let Some(inscribe_manifest) = find_inscribe_manifest() {
        match run_command(
            Command::new("cargo")
                .arg("run")
                .arg("--manifest-path")
                .arg(inscribe_manifest.as_os_str())
                .arg("-p")
                .arg("inscribe-cli")
                .arg("--")
                .args(args),
            "cargo run -p inscribe-cli -- emit mlib",
        ) {
            Ok(()) => return Ok(()),
            Err(error) => failures.push(error),
        }
    } else {
        failures.push("could not find inscribe/Cargo.toml for cargo fallback".to_string());
    }

    Err(format!(
        "failed to emit mlib; {}. Set SUTURE_INSCRIBE_BIN to a valid inscribe executable.",
        failures.join("; ")
    ))
}

pub fn remove_cached_spool(dir: &Path, name: &str) -> Result<(), String> {
    let sources = suture_root(dir).join("sources").join(name);
    let mlibs = suture_root(dir).join("mlib").join(name);

    if sources.exists() {
        fs::remove_dir_all(&sources).map_err(|error| error.to_string())?;
    }
    if mlibs.exists() {
        fs::remove_dir_all(&mlibs).map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub fn host_target() -> &'static str {
    if cfg!(windows) {
        "windows-x86_64"
    } else {
        "linux-x86_64"
    }
}

fn find_inscribe_manifest() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let current_dir = std::env::current_dir().ok();
    let mut starts = Vec::new();
    if let Some(current_dir) = current_dir {
        starts.push(current_dir);
    }
    starts.push(manifest_dir);

    for start in starts {
        for ancestor in start.ancestors() {
            let candidate = ancestor.join("inscribe").join("Cargo.toml");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

fn run_git<const N: usize>(args: [&str; N], cwd: Option<&Path>) -> Result<(), String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    run_command(&mut command, "git")
}

fn run_command(command: &mut Command, label: &str) -> Result<(), String> {
    let status = command.status().map_err(|error| format!("{label}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed with status {status}"))
    }
}

fn as_arg(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("path `{}` is not valid UTF-8", path.display()))
}

#[allow(dead_code)]
fn _os_str_lossy(value: &OsStr) -> String {
    value.to_string_lossy().to_string()
}
