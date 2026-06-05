use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::config;
use crate::util::{self, e};
use anyhow::Result;

pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn run_backup(source: &str, dest: &str, full: bool, force_folders: &[String], keep_dir: bool, min_size_gb: u64, excludes: &[String]) -> Result<()> {
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

    if keep_dir {
        let dir_name = std::path::Path::new(&src_expanded)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "backup".to_string());
        let full_dst = format!("{}/{}", dest_expanded, dir_name);
        let mtime = util::dir_mtime(&src_expanded).unwrap_or(0);

        if !full && !force_folders.contains(&dir_name) && manifest.get(&dir_name) == Some(&mtime) {
            e(&format!("  {}{}{} unchanged", util::CYAN, dir_name, util::RESET));
            return Ok(());
        }

        if min_size_gb > 0 {
            let size = util::dir_size_gb(&src_expanded);
            if size < min_size_gb as f64 {
                e(&format!("  {}{}{} too small ({:.1}G < {}G), skipped", util::YELLOW, dir_name, util::RESET, size, min_size_gb));
                manifest.insert(dir_name, mtime);
                util::save_manifest(&manifest_path, &manifest)?;
                return Ok(());
            }
        }

        e(&format!("  {}{}{} → ...", util::BOLD, dir_name, util::RESET));
        util::copy_progress(&src_expanded, &full_dst, checkers, false, false)?;
        manifest.insert(dir_name, mtime);
        util::save_manifest(&manifest_path, &manifest)?;
        e("Done: 1 backed up");
        return Ok(());
    }

    let subdirs = util::list_subdirs(&src_expanded)?;

    let subdirs = if !excludes.is_empty() {
        subdirs.into_iter()
            .filter(|(name, _, _)| !excludes.contains(name))
            .collect()
    } else {
        subdirs
    };

    if subdirs.is_empty() {
        e("No subdirectories found, copying whole tree");
        util::copy_progress(source, dest, checkers, false, false)?;
        return util::save_manifest(&manifest_path, &manifest);
    }

    let mut changed = 0u32;
    let mut skipped = 0u32;

    for (name, full_src, mtime) in &subdirs {
        if INTERRUPTED.load(Ordering::SeqCst) {
            e(&format!("{}Interrupted, exiting{}", util::YELLOW, util::RESET));
            break;
        }

        if !full && !force_folders.contains(name) && manifest.get(name.as_str()) == Some(mtime) {
            e(&format!("  {}{}{} unchanged", util::CYAN, name, util::RESET));
            skipped += 1;
            continue;
        }
        let full_dst = format!("{}/{}", dest_expanded, name);
        e(&format!("  {}{}{} → ...", util::BOLD, name, util::RESET));
        if let Err(err) = util::copy_progress(full_src, &full_dst, checkers, false, false) {
            if INTERRUPTED.load(Ordering::SeqCst) {
                util::save_manifest(&manifest_path, &manifest)?;
                e(&format!("{}Interrupted, saved progress{}", util::YELLOW, util::RESET));
                return Ok(());
            }
            return Err(err);
        }
        manifest.insert(name.clone(), *mtime);
        if let Err(err) = util::save_manifest(&manifest_path, &manifest) {
            e(&format!("{}Warning: failed to save manifest: {}{}", util::YELLOW, err, util::RESET));
        }
        changed += 1;
    }

    util::save_manifest(&manifest_path, &manifest)?;
    e(&format!("Done: {} backed up, {} skipped", changed, skipped));
    Ok(())
}
