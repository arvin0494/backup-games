# backup-games

Backup and restore `~/Games` to `/mnt/HDD4T/GAMES` using `rclone` with progress display, hardware-tuned parallelism, and `fzf`-based restore selection.

## Install

**curl one-liner:**
```bash
curl -sSL https://github.com/arvin0494/backup-games/raw/main/install.sh | bash
```

**Manual:**
```bash
git clone https://github.com/arvin0494/backup-games.git
cd backup-games
cargo build --release
cp target/release/backup-games ~/.local/bin/
```

## Usage

```bash
backup-games                    # backup ~/Games → /mnt/HDD4T/GAMES
bckup-games                     # same via alias (added by install.sh)
backup-games -s ~/OtherGames    # backup custom source
backup-games /custom/dest       # backup to custom destination
backup-games -r                 # restore with fzf multi-select
backup-games -y                 # skip confirmation
```

## Config

`~/.config/backup-games/config`:
```
source=~/Games
dest=/mnt/HDD4T/GAMES
```

CLI flags `--source`/`-s` and positional `dest` override the config. Lines starting with `#` are ignored.

## Dependencies

- `rclone` — file copy with progress
- `gdu` — disk usage analyzer (optional)
- `fzf` — interactive restore selection

Installed automatically on supported package managers (`apt`, `pacman`, `dnf`, `zypper`).

## How it works

1. Estimates directory size in a parallel thread
2. Detects destination disk type (HDD=3 checkers, SSD=8, NVMe=16) for optimal rclone parallelism
3. Runs `rclone copy` with `--progress` and inherited stderr
4. Logs all messages to `/tmp/backup-games.log`

Restore scans the backup, pipes items into `fzf --multi`, and copies selected items back.
