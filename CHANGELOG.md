# Changelog

## v2.2.0 — 2026-07-15

- **Recursive mtime**: `dir_mtime` now walks the full directory tree to detect deep file changes (e.g. game updates that modify files inside subdirectories without touching the top-level mtime)
- **`-r --full`**: restore all items without fzf selection
- **Refactor**: `CopyOpts` builder replaces 8-param `copy_progress` signature; deduplicated exclude logic in backup

## v2.1.3 — 2026-06-06

- **Subdirectory restore**: when restoring a dirsource, auto-detect single subdir or fzf-pick multiple, preserving relative paths
- **backup_exclude config**: rclone `--exclude` patterns (e.g. `webCaches/`) skipped during backup and restore

## v2.1.2 — 2025-06-06

- **Restore without --update**: restore now overwrites local files completely, preventing game update prompts after restore

## v2.1.1 — 2025-06-05

- **Remove automatic update prompt**: `--check-update` still works manually
- **Prune safety**: `rclone delete` instead of `purge` to preserve dirsource subdirs
- **Exclude absolute paths**: supports `~/Games/honkers-railway-launcher` in exclude
- **Fix**: don't copy whole tree when all subdirs excluded
- **Prune order**: deleted after backup completes (safe from interruption)

## v2.1.0 — 2025-06-05

- **--update** flag added to rclone: skip files newer on remote (multi-device safety)
- **Daily update check**: only prompts once per day, `--check-update` forces it
- **exclude config key**: skip subdirectories in flat sources (name or absolute path)
- **dirsources relative paths**: preserves path structure under ~/Games for proper restore
- **Prune**: excluded directories are cleaned from destination after backup
- **Chown**: destination files owned to user after copy (external drive fix)
- **Uninstall script**: `uninstall.sh` or `install.sh --uninstall`
- **Config always updated**: installer overwrites config with latest defaults (old backed up)

## v0.1.0 — 2025-05-30

- Initial release
- Flat and directory sources
- Change tracking via manifest
- Ctrl+C graceful handling
- fzf-based restore
- Hardware-tuned rclone parallelism
