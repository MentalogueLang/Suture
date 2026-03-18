use crate::support::{
    built_mlib_path, emit_mlib, find_exact_spool, load_manifest, project_dir, resolve_build_entry,
    source_checkout_dir, sync_spool_source,
};

pub fn run(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("usage: suture pull".to_string());
    }

    let dir = project_dir()?;
    let manifest = load_manifest(&dir)?;
    if manifest.dependencies.is_empty() {
        println!("no dependencies to pull");
        return Ok(());
    }

    for (name, version) in &manifest.dependencies {
        let entry = find_exact_spool(name, version)?;
        let source_dir = source_checkout_dir(&dir, &entry.name, &entry.version);
        sync_spool_source(&entry, &source_dir)?;

        let entry_source = resolve_build_entry(&entry, &source_dir)?;
        let output = built_mlib_path(&dir, &entry.name, &entry.version);
        emit_mlib(&entry_source, &output)?;

        println!(
            "pulled {} {} -> {}",
            entry.name,
            entry.version,
            output.display()
        );
    }

    Ok(())
}
