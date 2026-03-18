use crate::support::{
    built_mlib_path, find_exact_spool, load_manifest, project_dir, resolve_build_entry,
    source_checkout_dir,
};

pub fn run(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("usage: suture audit".to_string());
    }

    let dir = project_dir()?;
    let manifest = load_manifest(&dir)?;
    if manifest.dependencies.is_empty() {
        println!("no dependencies declared");
        return Ok(());
    }

    let mut failures = Vec::new();

    for (name, version) in &manifest.dependencies {
        match find_exact_spool(name, version) {
            Ok(entry) => {
                println!("ok index {} {}", entry.name, entry.version);

                let source_dir = source_checkout_dir(&dir, &entry.name, &entry.version);
                if source_dir.exists() {
                    println!("ok source {}", source_dir.display());
                } else {
                    failures.push(format!(
                        "missing source checkout for {} {} at {}",
                        entry.name,
                        entry.version,
                        source_dir.display()
                    ));
                }

                match resolve_build_entry(&entry, &source_dir) {
                    Ok(build_entry) => println!("ok entry {}", build_entry.display()),
                    Err(error) => failures.push(error),
                }

                let mlib = built_mlib_path(&dir, &entry.name, &entry.version);
                if mlib.exists() {
                    println!("ok mlib {}", mlib.display());
                } else {
                    failures.push(format!(
                        "missing mlib for {} {} at {}",
                        entry.name,
                        entry.version,
                        mlib.display()
                    ));
                }
            }
            Err(error) => failures.push(error),
        }
    }

    if failures.is_empty() {
        println!("audit passed");
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("audit: {failure}");
        }
        Err(format!("audit failed with {} issue(s)", failures.len()))
    }
}
