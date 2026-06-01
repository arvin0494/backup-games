use crate::util::{self, e};
use anyhow::Result;

pub fn run_backup(source: &str, dest: &str, _yes: bool) -> Result<()> {
    e(&format!("Starting backup: {} → {}", source, dest));

    let src = source.to_string();
    let est = std::thread::spawn(move || {
        if let Ok(size) = util::run(&format!("du -sh {} 2>/dev/null || true", util::expand_tilde(&src))) {
            e(&format!("Estimated size: {}{}{}", util::CYAN, size, util::RESET));
        }
    });

    let checkers = util::detect_checkers(dest);
    let kind = if checkers <= 3 {
        "HDD"
    } else if checkers <= 8 {
        "SSD"
    } else {
        "NVMe"
    };
    e(&format!("Checkers: {} ({})", checkers, kind));

    let _ = est.join();

    e("Copying files...");
    util::copy_progress(source, dest, checkers, false, false)?;

    Ok(())
}
