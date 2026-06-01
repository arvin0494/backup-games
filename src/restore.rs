use crate::util::{self, e};
use anyhow::{Result, Context};
use std::fs;
use std::path::Path;

pub fn run_restore(source: &str, dest: &str) -> Result<()> {
    e(&format!("Scanning backup: {}", source));

    let src = util::expand_tilde(source);
    let src_path = Path::new(&src);
    if !src_path.exists() {
        anyhow::bail!("Backup source not found: {}", src);
    }

    let items = util::run(&format!("find \"{}\" -mindepth 1 -maxdepth 1 | sort", src))
        .context("Failed to scan backup directory")?;

    let lines: Vec<&str> = items.lines().filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        anyhow::bail!("No items found in backup");
    }

    e(&format!("Found {} item(s).", lines.len()));

    let item_file = "/tmp/backup-games-items.txt";
    let sel_file = "/tmp/backup-games-selection.txt";
    fs::write(item_file, &items)?;

    util::run_ok(&format!(
        "fzf --multi --prompt='Select to restore > ' < {} > {}",
        item_file, sel_file
    ))?;

    let selected = fs::read_to_string(sel_file)?;
    let selections: Vec<&str> = selected.lines().filter(|l| !l.is_empty()).collect();

    if selections.is_empty() {
        e("No items selected.");
        return Ok(());
    }

    e(&format!("Restoring {} item(s)...", selections.len()));
    for item in &selections {
        let item_name = Path::new(item)
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        let dest_item = format!("{}/{}", dest, item_name);
        e(&format!("  {item_name} → {dest_item}"));
        util::copy_progress(item, &dest_item, 4, false, false)?;
    }

    Ok(())
}
