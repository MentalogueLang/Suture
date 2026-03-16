use std::fs;
use suture_index::{list_spools, spools_root, SpoolEntry};

pub fn run(args: &[String]) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        return Err(usage());
    };

    match command.as_str() {
        "add" => add(rest),
        "list" => list(),
        other => Err(format!("unknown spool command `{other}`\n\n{}", usage())),
    }
}

fn usage() -> String {
    [
        "usage:",
        "  suture spool add <name> <version> <git-url> [--tag <tag>]",
        "  suture spool list",
    ]
    .join("\n")
}

fn add(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err(usage());
    }

    let name = args[0].trim();
    let version = args[1].trim().trim_start_matches('v');
    let git = args[2].trim();
    if name.is_empty() || version.is_empty() || git.is_empty() {
        return Err("name, version, and git url are required".to_string());
    }

    let mut tag = None;
    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--tag" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value after --tag".to_string());
                };
                tag = Some(value.trim().to_string());
            }
            other => return Err(format!("unknown flag `{other}`")),
        }
        index += 1;
    }

    let root = spools_root().map_err(|error| error.to_string())?;
    if !root.exists() {
        return Err(format!(
            "spools registry not found at `{}`",
            root.display()
        ));
    }

    let spool_path = root.join("entries").join(name).join(version);
    fs::create_dir_all(&spool_path).map_err(|error| error.to_string())?;

    let file_path = spool_path.join("spool.toml");
    let entry = SpoolEntry::from_parts(name, version, git, tag.as_deref());
    fs::write(&file_path, entry.to_toml()).map_err(|error| error.to_string())?;
    println!("wrote {}", file_path.display());
    Ok(())
}

fn list() -> Result<(), String> {
    let entries = list_spools().map_err(|error| error.to_string())?;
    if entries.is_empty() {
        println!("no spools found");
        return Ok(());
    }
    for entry in entries {
        println!("{} {}", entry.name, entry.version);
    }
    Ok(())
}
