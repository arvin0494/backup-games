use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use anyhow::{Context, Result, bail};

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

static LOG_FILE: Mutex<Option<fs::File>> = Mutex::new(None);

pub fn init_log(path: &str) {
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .ok();
    *LOG_FILE.lock().unwrap() = file;
}

pub fn e(msg: &str) {
    println!("{BOLD}[{GREEN}*{RESET}{BOLD}]{RESET} {msg}");
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut f) = *guard {
            let _ = writeln!(f, "{msg}");
        }
    }
}

pub fn run(cmd: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn command")?
        .wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Command failed: `{cmd}`\n{stderr}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn run_ok(cmd: &str) -> Result<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn()
        .context("Failed to spawn command")?
        .wait()?;
    if !status.success() {
        bail!("Command exited with code {:?}", status.code());
    }
    Ok(())
}

pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]).to_string_lossy().to_string();
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.to_string_lossy().to_string();
        }
    }
    path.to_string()
}

pub fn detect_checkers(dest: &str) -> u32 {
    let dest_expanded = expand_tilde(dest);
    let dev = match run(&format!(
        "df --output=source {} 2>/dev/null | tail -1",
        dest_expanded
    )) {
        Ok(d) => d.trim().to_string(),
        Err(_) => return 8,
    };
    if dev.is_empty() || dev == "Filesystem" || dev.starts_with("tmpfs") {
        return 8;
    }

    let parent = run(&format!("lsblk -no pkname {} 2>/dev/null", dev))
        .ok()
        .map(|p| p.trim().to_string())
        .unwrap_or_else(|| dev.trim_start_matches("/dev/").to_string());

    if parent.starts_with("nvme") {
        return 16;
    }

    if let Ok(content) = fs::read_to_string(format!("/sys/block/{}/queue/rotational", parent)) {
        if content.trim() == "1" {
            return 3;
        }
    }

    8
}

#[allow(dead_code)]
pub fn detect_path(base: &str, suffix: &str) -> String {
    let hostname = run("hostname -s").unwrap_or_else(|_| "unknown".into());
    let distro = get_os_id();
    format!("{}/{}-{}/{}", base, hostname, distro, suffix)
}

#[allow(dead_code)]
fn get_os_id() -> String {
    let content = match fs::read_to_string("/etc/os-release") {
        Ok(c) => c,
        Err(_) => return "linux".into(),
    };
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("ID=") {
            return val.trim_matches('"').to_string();
        }
    }
    "linux".into()
}

pub fn copy_progress(src: &str, dst: &str, checkers: u32, _ntfs: bool, skip_links: bool) -> Result<()> {
    let src_expanded = expand_tilde(src);
    let dst_expanded = expand_tilde(dst);

    let mut args = vec![
        "copy".to_string(),
        src_expanded.clone(),
        dst_expanded.clone(),
        "--progress".to_string(),
        format!("--checkers={}", checkers),
        "--verbose".to_string(),
    ];

    if skip_links {
        args.push("--skip-links".to_string());
    }

    fs::create_dir_all(&dst_expanded).context("Failed to create destination directory")?;

    let mut child = Command::new("rclone")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to spawn rclone. Is rclone installed?")?;

    let status = child.wait()?;
    if !status.success() {
        bail!("rclone copy failed");
    }
    Ok(())
}

pub fn install_deps() -> Result<()> {
    let deps = ["rclone", "gdu", "fzf"];
    let missing: Vec<_> = deps
        .iter()
        .filter(|bin| {
            Command::new("sh")
                .arg("-c")
                .arg(format!("which {}", bin))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| !s.success())
                .unwrap_or(true)
        })
        .copied()
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    e(&format!("Installing missing dependencies: {}", missing.join(", ")));

    let pm = detect_pm();
    let install_cmd = match pm.as_str() {
        "apt" => format!("sudo apt install -y {}", missing.join(" ")),
        "pacman" => format!("sudo pacman -S --noconfirm {}", missing.join(" ")),
        "dnf" => format!("sudo dnf install -y {}", missing.join(" ")),
        "zypper" => format!("sudo zypper install -y {}", missing.join(" ")),
        _ => bail!(
            "Unsupported package manager. Install manually: {}",
            missing.join(", ")
        ),
    };

    run_ok(&install_cmd)?;
    Ok(())
}

fn detect_pm() -> String {
    for (bin, name) in [
        ("apt-get", "apt"),
        ("pacman", "pacman"),
        ("dnf", "dnf"),
        ("zypper", "zypper"),
    ] {
        if Command::new("sh")
            .arg("-c")
            .arg(format!("which {}", bin))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return name.to_string();
        }
    }
    "unknown".to_string()
}
