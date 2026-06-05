use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::SystemTime;
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

pub fn copy_progress(src: &str, dst: &str, checkers: u32, ntfs: bool, skip_links: bool) -> Result<()> {
    let src_expanded = expand_tilde(src);
    let dst_expanded = expand_tilde(dst);

    let mut args = vec![
        "copy".to_string(),
        src_expanded.clone(),
        dst_expanded.clone(),
        "--progress".to_string(),
        "--stats=1s".to_string(),
        format!("--checkers={}", checkers),
        format!("--transfers={}", checkers),
        "--update".to_string(),
        "--verbose".to_string(),
    ];

    if skip_links {
        args.push("--skip-links".to_string());
    }
    if ntfs {
        args.push("--ignore-errors".to_string());
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

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("sudo chown -R --dereference $(id -u):$(id -g) \"{}\" 2>/dev/null", &dst_expanded))
        .status();

    Ok(())
}

pub fn dir_mtime(path: &str) -> Option<u64> {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

pub fn load_manifest(path: &str) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    let content = fs::read_to_string(path).unwrap_or_default();
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if let Ok(ts) = v.trim().parse::<u64>() {
                map.insert(k.trim().to_string(), ts);
            }
        }
    }
    map
}

pub fn save_manifest(path: &str, map: &HashMap<String, u64>) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    let mut s = String::new();
    let mut pairs: Vec<_> = map.iter().collect();
    pairs.sort_by_key(|p| p.0.clone());
    for (k, v) in pairs {
        s.push_str(&format!("{}={}\n", k, v));
    }
    fs::write(path, s)?;
    Ok(())
}

pub fn list_subdirs(path: &str) -> Result<Vec<(String, String, u64)>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let full = path.to_string_lossy().to_string();
                let mtime = dir_mtime(&full).unwrap_or(0);
                dirs.push((name.to_string(), full, mtime));
            }
        }
    }
    dirs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(dirs)
}

pub fn dir_size_gb(path: &str) -> f64 {
    let out = run(&format!("du -sb {} 2>/dev/null | cut -f1", path)).unwrap_or_default();
    let bytes: f64 = out.parse().unwrap_or(0.0);
    bytes / (1073741824.0) // bytes → GB
}

pub fn install_deps() -> Result<()> {
    let deps = ["rclone", "gdu", "fzf", "jq"];
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
