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

    let source = cli
        .source
        .clone()
        .or_else(|| user_cfg.get("source").cloned())
        .unwrap_or_else(|| config::DEFAULT_SOURCE.to_string());

    let dest = cli
        .dest
        .clone()
        .or_else(|| user_cfg.get("dest").cloned())
        .unwrap_or_else(|| config::DEFAULT_DEST.to_string());

    if cli.check_update {
        check_update();
        return;
    }

    check_update();

    if let Err(e) = util::install_deps() {
        util::e(&format!("{}Failed to install deps: {}{}", util::RED, e, util::RESET));
        std::process::exit(1);
    }

    let op = if cli.restore {
        "restore" as &str
    } else {
        "backup" as &str
    };
    util::e(&format!("Mode: {op}, source: {source}, dest: {dest}"));

    let result = panic::catch_unwind(|| {
        if cli.restore {
            restore::run_restore(&source, &dest)
        } else {
            backup::run_backup(&source, &dest, cli.full)
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

    match latest {
        Ok(tag) if !tag.is_empty() => {
            let on_tag = VERSION == tag || VERSION.starts_with(&format!("{}-", tag));
            if on_tag {
                util::e(&format!(
                    "{}{} up to date ({}){}",
                    util::GREEN, VERSION, tag, util::RESET
                ));
            } else {
                util::e(&format!(
                    "{}Update available: {} → {}{}",
                    util::BOLD, VERSION, tag, util::RESET
                ));
                util::e("Run: curl -sSL https://github.com/arvin0494/backup-games/raw/main/install.sh | bash");
            }
        }
        _ => {
            util::e(&format!("{}Could not check for updates (no network or missing jq?){}", util::RED, util::RESET));
        }
    }
}
