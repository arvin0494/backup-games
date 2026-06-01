use std::collections::HashMap;
use crate::config;
use crate::util::{self, e};
use anyhow::Result;

pub fn run_backup(source: &str, dest: &str, full: bool) -> Result<()> {
    e(&format!("Starting backup: {} → {}", source, dest));

    let src_expanded = util::expand_tilde(source);
    let dest_expanded = util::expand_tilde(dest);

    let src = source.to_string();
    let est = std::thread::spawn(move || {
        if let Ok(size) = util::run(&format!("du -sh {} 2>/dev/null || true", util::expand_tilde(&src))) {
            e(&format!("Estimated size: {}{}{}", util::CYAN, size, util::RESET));
        }
    });

    let checkers = util::detect_checkers(dest);
    let kind = if checkers <= 3 { "HDD" } else if checkers <= 8 { "SSD" } else { "NVMe" };
    e(&format!("Checkers: {} ({})", checkers, kind));
    let _ = est.join();

    let manifest_path = util::expand_tilde(config::MANIFEST_FILE);
    let mut manifest: HashMap<String, u64> = if full {
        e("Full backup requested, ignoring manifest");
        HashMap::new()
    } else {
        util::load_manifest(&manifest_path)
    };

    let subdirs = util::list_subdirs(&src_expanded)?;
    if subdirs.is_empty() {
        e("No subdirectories found, copying whole tree");
        util::copy_progress(source, dest, checkers, false, false)?;
        return save_manifest(&manifest_path, &manifest);
    }

    let mut changed = 0u32;
    let mut skipped = 0u32;

    for (name, full_src, mtime) in &subdirs {
        if !full && manifest.get(name.as_str()) == Some(mtime) {
            e(&format!("  {}{}{} unchanged", util::CYAN, name, util::RESET));
            skipped += 1;
            continue;
        }
        let full_dst = format!("{}/{}", dest_expanded, name);
        e(&format!("  {}{}{} → ...", util::BOLD, name, util::RESET));
        util::copy_progress(full_src, &full_dst, checkers, false, false)?;
        manifest.insert(name.clone(), *mtime);
        changed += 1;
    }

    save_manifest(&manifest_path, &manifest)?;
    e(&format!("Done: {} backed up, {} skipped", changed, skipped));
    Ok(())
}

fn save_manifest(path: &str, map: &HashMap<String, u64>) -> Result<()> {
    util::save_manifest(path, map)
}
