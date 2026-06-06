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
backup-games -y                 # skip confirmation (unused currently)
backup-games --full             # force full backup, ignore change tracking
backup-games --check-update     # check GitHub for newer version
backup-games --version          # show version (git describe)
```

## Config

`~/.config/backup-games/config`:
```
sources=~/Games
dirsources=~/Games/honkers-railway-launcher/HSR,~/.local/share/honkers
exclude=~/Games/honkers-railway-launcher
dest=/mnt/HDD4T/GAMES
```

- `sources` — subdirectories are flattened into the destination root
- `dirsources` — each source is copied as a whole directory keeping its own name
- `source` — backward compat, single flat source
- `min_size` — skip `dirsources` smaller than this many GB (0 = no limit)
- `exclude` — comma-separated dir names or paths to skip in flat sources
- `backup_exclude` — rclone `--exclude` patterns applied during both backup and restore (e.g. `webCaches/`)
- CLI flag `--source`/`-s` overrides all config sources

## Game-specific notes

### Honkai Star Rail (honkers-railway-launcher)

The launcher stores machine-specific browser cache inside its `HSR/StarRail_Data/webCaches/` directory. Restoring this from another PC causes the launcher to detect content mismatches and re-download the game (~10 GB).

**Fix:** add to your config:
```
backup_exclude = webCaches/
```
This skips `webCaches/` (at any depth) during backup and restore. Each PC keeps its own cache, and the actual game data remains identical. After adding, run a fresh backup to purge it from the destination.

## Dependencies

- `rclone` — file copy with progress
- `gdu` — disk usage analyzer (optional)
- `fzf` — interactive restore selection

Installed automatically on supported package managers (`apt`, `pacman`, `dnf`, `zypper`).

## How it works

1. Estimates directory size in a parallel thread
2. Detects destination disk type (HDD=3 checkers, SSD=8, NVMe=16) for optimal rclone parallelism
3. **Change tracking** — each game directory's modification time is stored in `~/.local/share/backup-games/manifest`. Unchanged directories are skipped entirely, saving the scan overhead.
4. Runs `rclone copy` only on changed directories with `--progress` and inherited stderr
5. Logs all messages to `/tmp/backup-games.log`

Restore scans the backup, pipes items into `fzf --multi`, and copies selected items back.
