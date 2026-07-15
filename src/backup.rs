use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::config;
use crate::util::{self, e, CopyOpts};
use anyhow::Result;
use std::path::Path;

pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);

fn is_excluded(name: &str, full_src: &str, excludes: &[String]) -> bool {
    excludes.iter().any(|e| e == name || util::expand_tilde(e) == full_src)
}

fn run_source_backup(
    src: &str,
    dest: &str,
    full: bool,
    force_folders: &[String],
    keep_dir: bool,
    min_size_gb: u64,
    excludes: &[String],
    backup_exclude: &[String],
) -> Result<usize> {
    let src_expanded = util::expand_tilde(src);
    let dest_expanded = util::expand_tilde(dest);
    let checkers = util::detect_checkers(dest);
    let manifest_path = util::expand_tilde(config::MANIFEST_FILE);
    let mut manifest: HashMap<String, u64> = if full {
        HashMap::new()
    } else {
        util::load_manifest(&manifest_path)
    };

    if keep_dir {
        let games_prefix = util::expand_tilde("~/Games");
        let dir_name = if src_expanded.starts_with(&games_prefix) {
            src_expanded[games_prefix.len()..].trim_start_matches('/').to_string()
        } else {
            Path::new(&src_expanded)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "backup".to_string())
        };
        let full_dst = format!("{}/{}", dest_expanded, dir_name);
        let mtime = util::dir_mtime(&src_expanded).unwrap_or(0);

        // Check if the backup destination actually has files.
        // If the flat source pruned this directory, the manifest may still
        // show it as unchanged even though the files are gone.
        let dst_has_files = Path::new(&full_dst).exists() && {
            util::run(&format!(
                "find \"{}\" -type f 2>/dev/null | head -1", full_dst
            )).map(|s| !s.trim().is_empty()).unwrap_or(false)
        };

        if !full
            && !force_folders.contains(&dir_name)
            && manifest.get(&dir_name) == Some(&mtime)
            && dst_has_files
        {
            e(&format!("  {}{}{} unchanged", util::CYAN, dir_name, util::RESET));
            return Ok(0);
        }

        if min_size_gb > 0 {
            let size = util::dir_size_gb(&src_expanded);
            if size < min_size_gb as f64 {
                e(&format!("  {}{}{} too small ({:.1}G < {}G), skipped", util::YELLOW, dir_name, util::RESET, size, min_size_gb));
                manifest.insert(dir_name, mtime);
                util::save_manifest(&manifest_path, &manifest)?;
                return Ok(0);
            }
        }

        e(&format!("  {}{}{} → ...", util::BOLD, dir_name, util::RESET));
        util::copy_progress(&CopyOpts::new(&src_expanded, &full_dst).checkers(checkers).update(true).exclude(backup_exclude))?;
        manifest.insert(dir_name, mtime);
        util::save_manifest(&manifest_path, &manifest)?;
        return Ok(1);
    }

    let subdirs = util::list_subdirs(&src_expanded)?;

    let excluded_names: Vec<String> = subdirs.iter()
        .filter(|(name, full_src, _)| is_excluded(name, full_src, excludes))
        .map(|(name, _, _)| name.clone())
        .collect();

    let subdirs: Vec<_> = subdirs.into_iter()
        .filter(|(name, full_src, _)| !is_excluded(name, full_src, excludes))
        .collect();

    if subdirs.is_empty() && excluded_names.is_empty() {
        e("No subdirectories found, copying whole tree");
        util::copy_progress(&CopyOpts::new(src, dest).checkers(checkers).update(true).exclude(backup_exclude))?;
        util::save_manifest(&manifest_path, &manifest)?;
        return Ok(1);
    }

    if subdirs.is_empty() {
        e("All subdirectories excluded, nothing to back up");
        return Ok(0);
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
        if let Err(err) = util::copy_progress(&CopyOpts::new(full_src, &full_dst).checkers(checkers).update(true).exclude(backup_exclude)) {
            if INTERRUPTED.load(Ordering::SeqCst) {
                util::save_manifest(&manifest_path, &manifest)?;
                e(&format!("{}Interrupted, saved progress{}", util::YELLOW, util::RESET));
                return Ok(changed as usize);
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

    for name in &excluded_names {
        let stale = format!("{}/{}", dest_expanded, name);
        if Path::new(&stale).exists() {
            e(&format!("  pruning stale: {}...", name));
            let _ = util::run(&format!("rclone delete \"{}\" 2>/dev/null", stale));
            e(&format!("  {}{}{} pruned", util::YELLOW, name, util::RESET));
        }
    }

    e(&format!("Done: {} backed up, {} skipped", changed, skipped));
    Ok(changed as usize)
}

pub fn run_backup(source: &str, dest: &str, full: bool, force_folders: &[String], keep_dir: bool, min_size_gb: u64, excludes: &[String], backup_exclude: &[String]) -> Result<()> {
    e(&format!("Starting backup: {} → {}", source, dest));

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

    if full {
        e("Full backup requested, ignoring manifest");
    }

    run_source_backup(source, dest, full, force_folders, keep_dir, min_size_gb, excludes, backup_exclude)?;
    Ok(())
}
