use crate::util::{self, e, CopyOpts};
use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

fn pick_subdir(backup_item: &str) -> Result<Option<String>> {
    let subdirs = util::run(&format!(
        "find \"{}\" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | sort", backup_item
    ))?;
    let lines: Vec<&str> = subdirs.lines().filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return Ok(None);
    }
    // When there are multiple subdirectories, restore the whole directory
    // instead of picking a single one (e.g. NTE has Client/ + NTEGlobal/).
    if lines.len() > 1 {
        return Ok(None);
    }
    let single = lines[0].trim();
    e(&format!("  detected single subdirectory: {}", Path::new(single).file_name().unwrap_or_default().to_string_lossy()));
    Ok(Some(single.to_string()))
}

fn restore_items(items: &[String], backup_root: &Path, restore_dest: &str, restore_exclude: &[String], force: bool) -> Result<()> {
    for item in items {
        let item_path = Path::new(item);
        let rel = item_path.strip_prefix(backup_root).unwrap_or(item_path);
        let dest = format!("{}/{}", restore_dest, rel.display());

        let (src_path, display_path, dest_path) = if let Some(sub) = pick_subdir(item)? {
            let sub_path = Path::new(&sub);
            let sub_rel = sub_path.strip_prefix(backup_root).unwrap_or(sub_path);
            let sub_dest = format!("{}/{}", restore_dest, sub_rel.display());
            e(&format!("  selecting subdirectory: {}", sub_rel.display()));
            (sub.clone(), sub_rel.display().to_string(), sub_dest)
        } else {
            (item.clone(), rel.display().to_string(), dest)
        };

        // Diagnose: check source exists and show file count
        let src_check = if Path::new(&src_path).exists() {
            let count = util::run(&format!(
                "find \"{}\" -type f 2>/dev/null | wc -l", src_path
            )).unwrap_or_else(|_| "?".to_string());
            let size = util::run(&format!(
                "du -sh \"{}\" 2>/dev/null | cut -f1", src_path
            )).unwrap_or_else(|_| "?".to_string());
            format!("{} files, {}", count.trim(), size.trim())
        } else {
            format!("{}DOES NOT EXIST{}", util::RED, util::RESET)
        };
        e(&format!("  {} {}→{}  ({})", display_path, util::BOLD, util::RESET, src_check));

        let mut opts = CopyOpts::new(&src_path, &dest_path).exclude(restore_exclude);
        if force { opts = opts.force(true); }
        util::copy_progress(&opts)?;
    }
    Ok(())
}

pub fn run_restore(backup_dest: &str, restore_exclude: &[String], full: bool, force: bool) -> Result<()> {
    let backup_dest = util::expand_tilde(backup_dest);
    let backup_root = Path::new(&backup_dest);

    if !backup_root.exists() {
        e(&format!("{}Backup destination not found: {}{}", util::RED, backup_dest, util::RESET));
        return Ok(());
    }

    let default_restore = util::expand_tilde("~/Games");
    print!("{}Restore to{} [{}]: ", util::YELLOW, util::RESET, default_restore);
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let restore_dest = if input.trim().is_empty() {
        default_restore
    } else {
        util::expand_tilde(input.trim())
    };

    let items = util::run(&format!("find \"{}\" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | sort", backup_dest))?;
    let lines: Vec<&str> = items.lines().filter(|l| !l.is_empty()).collect();

    if lines.is_empty() {
        e(&format!("{}No backups found in {}{}", util::YELLOW, backup_dest, util::RESET));
        return Ok(());
    }

    e(&format!("Found {} backup(s) in {}", lines.len(), backup_dest));

    if force {
        e(&format!("{}Force mode: --ignore-times will be passed to rclone{}", util::YELLOW, util::RESET));
    }

    if full {
        e(&format!("{}Full restore — restoring all items{}", util::GREEN, util::RESET));
        let all_items: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        return restore_items(&all_items, backup_root, &restore_dest, restore_exclude, force);
    }

    let item_file = "/tmp/backup-games-items.txt";
    let sel_file = "/tmp/backup-games-selection.txt";
    fs::write(item_file, &items)?;

    util::run_ok(&format!(
        "fzf --multi --prompt='Select items to restore > ' < {} > {}",
        item_file, sel_file
    ))?;

    let selected = fs::read_to_string(sel_file)?;
    let selections: Vec<&str> = selected.lines().filter(|l| !l.is_empty()).collect();

    if selections.is_empty() {
        e("No items selected");
        return Ok(());
    }

    e(&format!("Restoring {} item(s) to {}...", selections.len(), restore_dest));
    let sel_owned: Vec<String> = selections.iter().map(|s| s.to_string()).collect();
    restore_items(&sel_owned, backup_root, &restore_dest, restore_exclude, force)?;

    Ok(())
}
