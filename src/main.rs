use clap::Parser;
use std::panic;
use std::sync::atomic::Ordering;

mod config;
mod util;
mod backup;
mod restore;

pub const VERSION: &str = env!("BUILD_VERSION");

#[derive(Parser)]
#[command(name = "backup-games", about = "Backup and restore ~/Games", version = VERSION)]
struct Cli {
    #[arg(short = 'b', long = "backup")]
    backup: bool,

    #[arg(short = 'r', long = "restore")]
    restore: bool,

    #[arg(long = "check-update")]
    check_update: bool,

    #[arg(long = "full")]
    full: bool,

    #[arg(short = 'y', long = "yes")]
    yes: bool,

    #[arg(short = 's', long = "source")]
    source: Option<String>,

    #[arg(short = 'f', long = "force-folder")]
    force_folder: Vec<String>,

    dest: Option<String>,
}

fn main() {
    ctrlc::set_handler(move || {
        backup::INTERRUPTED.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    util::init_log(config::LOG_FILE);

    let cli = Cli::parse();

    let user_cfg = config::load_user_config();

    let sources = config::load_sources(&user_cfg, cli.source.clone());
    let dir_sources = config::load_dir_sources(&user_cfg);
    let min_size_gb = config::load_min_size_gb(&user_cfg);

    let dest = cli
        .dest
        .clone()
        .or_else(|| user_cfg.get("dest").cloned())
        .unwrap_or_else(|| config::DEFAULT_DEST.to_string());

    if cli.check_update {
        check_update();
        return;
    }

    let last_check_path = util::expand_tilde("~/.local/share/backup-games/last-update-check");
    let should_check = std::fs::read_to_string(&last_check_path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|ts| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now - ts > 86400
        })
        .unwrap_or(true);

    if should_check {
        check_update();
        if let Some(parent) = std::path::Path::new(&last_check_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = std::fs::write(&last_check_path, now.to_string());
    }

    if let Err(e) = util::install_deps() {
        util::e(&format!("{}Failed to install deps: {}{}", util::RED, e, util::RESET));
        std::process::exit(1);
    }

    let result = panic::catch_unwind(|| {
        if cli.restore {
            restore::run_restore(&dest)
        } else {
            for source in &sources {
                backup::run_backup(source, &dest, cli.full, &cli.force_folder, false, 0)?;
            }
            for source in &dir_sources {
                backup::run_backup(source, &dest, cli.full, &cli.force_folder, true, min_size_gb)?;
            }
            Ok(())
        }
    });

    match result {
        Ok(Ok(())) => util::e(&format!("{}{}Done!{}", util::BOLD, util::GREEN, util::RESET)),
        Ok(Err(e)) => {
            util::e(&format!("{}{}Error:{} {}", util::BOLD, util::RED, util::RESET, e));
            std::process::exit(1);
        }
        Err(_) => {
            util::e(&format!("{}{}Cancelled{}", util::BOLD, util::YELLOW, util::RESET));
            std::process::exit(1);
        }
    }
}

fn check_update() {
    let latest = util::run(
        "curl -sL 'https://api.github.com/repos/arvin0494/backup-games/tags?per_page=1' | \
         jq -r '.[0].name // empty'",
    );

    let tag = match latest {
        Ok(t) if !t.is_empty() => t,
        _ => {
            util::e(&format!("{}Could not check for updates{}", util::RED, util::RESET));
            return;
        }
    };

    let on_tag = VERSION == tag || VERSION.starts_with(&format!("{}-", tag));
    if on_tag {
        util::e(&format!("{}{} up to date{}", util::GREEN, VERSION, util::RESET));
        return;
    }

    util::e(&format!("{}Update available: {} → {}{}", util::BOLD, VERSION, tag, util::RESET));
    print!("{}Install now? [Y/n]{} ", util::YELLOW, util::RESET);
    let _ = std::io::Write::flush(&mut std::io::stdout());

    let mut input = String::new();
    let proceed = match std::io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y"),
        Err(_) => true,
    };

    if proceed {
        util::e("Installing update...");
        if let Err(e) = util::run_ok(
            "curl -sSL https://github.com/arvin0494/backup-games/raw/main/install.sh | bash",
        ) {
            util::e(&format!("{}Update failed: {}{}", util::RED, e, util::RESET));
        } else {
            util::e(&format!("{}Update complete, re-run the command{}", util::GREEN, util::RESET));
        }
        std::process::exit(0);
    }
}
