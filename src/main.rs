use clap::Parser;
use std::panic;

mod config;
mod util;
mod backup;
mod restore;

#[derive(Parser)]
#[command(name = "backup-games", about = "Backup and restore ~/Games")]
struct Cli {
    #[arg(short = 'b', long = "backup")]
    backup: bool,

    #[arg(short = 'r', long = "restore")]
    restore: bool,

    #[arg(short = 'y', long = "yes")]
    yes: bool,

    #[arg(short = 's', long = "source")]
    source: Option<String>,

    dest: Option<String>,
}

fn main() {
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
            backup::run_backup(&source, &dest, cli.yes)
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
