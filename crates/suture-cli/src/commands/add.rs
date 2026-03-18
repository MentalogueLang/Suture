use crate::support::{find_exact_spool, load_manifest, project_dir, save_manifest};

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err("usage: suture add <spool> <version>".to_string());
    }

    let name = args[0].trim();
    let version = args[1].trim().trim_start_matches('v');
    if name.is_empty() || version.is_empty() {
        return Err("spool name and version are required".to_string());
    }

    let entry = find_exact_spool(name, version)?;
    let dir = project_dir()?;
    let mut manifest = load_manifest(&dir)?;
    manifest.set_dependency(entry.name.clone(), entry.version.clone());
    let path = save_manifest(&dir, &manifest)?;
    println!("added {} {} to {}", entry.name, entry.version, path.display());
    Ok(())
}
