# backup-games

Backup and restore `~/Games` to `/mnt/HDD4T/GAMES` using `rclone` with progress display, hardware-tuned parallelism, and `fzf`-based restore selection.

## Usage

```bash
backup-games              # backup ~/Games → /mnt/HDD4T/GAMES
backup-games -r           # restore with fzf multi-select
backup-games /custom/dest # backup to a custom destination
backup-games -y           # skip confirmation prompts
```

## Config

Optional user config at `~/.config/backup-games/config`:

```
source=~/Games
dest=/mnt/HDD4T/GAMES
```

Lines starting with `#` are ignored.

## Install

```bash
git clone https://github.com/arvin0494/backup-games.git
cd backup-games
cargo build --release
cp target/release/backup-games ~/.local/bin/
```

Or run `./install.sh` to install Rust, build, add shell alias, and create a default config.

## Dependencies

- `rclone` — file copy with progress
- `gdu` — disk usage analyzer (optional)
- `fzf` — interactive restore selection

Installed automatically via `install_deps()` on supported package managers (`apt`, `pacman`, `dnf`, `zypper`).

## How it works

1. Estimates directory size in a parallel thread
2. Detects destination disk type (HDD=3 checkers, SSD=8, NVMe=16) for optimal rclone parallelism
3. Runs `rclone copy` with `--progress` and inherited stderr
4. Logs all messages to `/tmp/backup-games.log`

Restore scans the backup directory, pipes items into `fzf --multi`, and copies selected items back.
