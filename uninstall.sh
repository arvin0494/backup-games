#!/usr/bin/env bash
set -euo pipefail

PROJECT="backup-games"
BIN="$HOME/.local/bin/$PROJECT"
CONFIG_DIR="$HOME/.config/$PROJECT"
DATA_DIR="$HOME/.local/share/$PROJECT"

R=$'\033[31m' G=$'\033[32m' Y=$'\033[33m' C=$'\033[36m' M=$'\033[35m'
B=$'\033[1m' D=$'\033[2m' N=$'\033[0m'

echo "${M}╔══════════════════════════════════════════════════════════════╗${N}"
echo "${M}║${N}               ${B}${C}backup-games — UNINSTALL${N}                ${M}║${N}"
echo "${M}╚══════════════════════════════════════════════════════════════╝${N}"

rem() { printf "${B}  ▸${N} %-24s ${R}removed${N}\n" "$1"; }
skip() { printf "${B}  ▸${N} %-24s ${D}not found${N}\n" "$1"; }

echo
printf "${D}  ┌──────────────────────────────────────────────────────┐${N}\n"
printf "${C}  │ [0x01] REMOVING BINARY                              ${N}\n"
printf "${D}  └──────────────────────────────────────────────────────┘${N}\n"
if [ -f "$BIN" ]; then
    rm -f "$BIN"
    rem "Binary" "$BIN"
else
    skip "Binary"
fi

echo
printf "${D}  ┌──────────────────────────────────────────────────────┐${N}\n"
printf "${C}  │ [0x02] REMOVING CONFIG & DATA                       ${N}\n"
printf "${D}  └──────────────────────────────────────────────────────┘${N}\n"
if [ -d "$CONFIG_DIR" ]; then
    rm -rf "$CONFIG_DIR"
    rem "Config" "$CONFIG_DIR"
else
    skip "Config"
fi
if [ -d "$DATA_DIR" ]; then
    rm -rf "$DATA_DIR"
    rem "Data" "$DATA_DIR"
else
    skip "Data"
fi

echo
printf "${D}  ┌──────────────────────────────────────────────────────┐${N}\n"
printf "${C}  │ [0x03] REMOVING SHELL ALIASES                       ${N}\n"
printf "${D}  └──────────────────────────────────────────────────────┘${N}\n"
aliases_removed=0
for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    [ -f "$rc" ] || continue
    if grep -q "alias $PROJECT=" "$rc" 2>/dev/null || grep -q "alias bckup-games=" "$rc" 2>/dev/null; then
        sed -i "/^alias $PROJECT=/d" "$rc"
        sed -i "/^alias bckup-games=/d" "$rc"
        aliases_removed=$((aliases_removed+1))
        rem "Aliases in $(basename "$rc")"
    fi
done
fish="$HOME/.config/fish/config.fish"
if [ -f "$fish" ]; then
    if grep -q "alias $PROJECT " "$fish" 2>/dev/null || grep -q "alias bckup-games " "$fish" 2>/dev/null; then
        sed -i "/^alias $PROJECT /d" "$fish"
        sed -i "/^alias bckup-games /d" "$fish"
        aliases_removed=$((aliases_removed+1))
        rem "Aliases in config.fish"
    fi
fi
[ "$aliases_removed" -eq 0 ] && skip "Aliases"

echo
printf "${D}  ────────────────────────────────────────────────────────${N}\n"
printf "\n  ${G}${B}✔ Uninstall complete${N}\n"
echo
