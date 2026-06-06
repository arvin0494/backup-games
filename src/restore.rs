use crate::util::{self, e};
use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn run_restore(backup_dest: &str) -> Result<()> {
    let backup_dest = util::expand_tilde(backup_dest);
    let backup_path = Path::new(&backup_dest);

    if !backup_path.exists() {
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

    let items = util::run(&format!("find \"{}\" -mindepth 1 -maxdepth 1 -type d | sort", backup_dest))?;
    let lines: Vec<&str> = items.lines().filter(|l| !l.is_empty()).collect();

    if lines.is_empty() {
        e(&format!("{}No backups found in {}{}", util::YELLOW, backup_dest, util::RESET));
        return Ok(());
    }

    e(&format!("Found {} backup(s) in {}", lines.len(), backup_dest));

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
    for item in &selections {
        let item_name = Path::new(item)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        let dest_item = format!("{}/{}", restore_dest, item_name);
        e(&format!("  {item_name} → {dest_item}"));
        util::copy_progress(item, &dest_item, 4, false, false, false)?;
    }

    Ok(())
}
