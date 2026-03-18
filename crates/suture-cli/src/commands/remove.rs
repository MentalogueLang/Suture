use crate::support::{load_manifest, project_dir, remove_cached_spool, save_manifest};

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 1 {
        return Err("usage: suture remove <spool>".to_string());
    }

    let name = args[0].trim();
    if name.is_empty() {
        return Err("spool name cannot be empty".to_string());
    }

    let dir = project_dir()?;
    let mut manifest = load_manifest(&dir)?;
    if !manifest.remove_dependency(name) {
        return Err(format!("spool `{name}` is not in suture.toml"));
    }

    let path = save_manifest(&dir, &manifest)?;
    remove_cached_spool(&dir, name)?;
    println!("removed {name} from {}", path.display());
    Ok(())
}
