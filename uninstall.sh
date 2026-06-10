#!/usr/bin/env bash
set -euo pipefail

PROJECT="backup-games"
BIN="$HOME/.local/bin/$PROJECT"
CONFIG_DIR="$HOME/.config/$PROJECT"
DATA_DIR="$HOME/.local/share/$PROJECT"

R=$'\033[31m' G=$'\033[32m' Y=$'\033[33m' C=$'\033[36m' B=$'\033[1m' D=$'\033[2m' N=$'\033[0m'

pacman_banner() {
    local text="$1"
    local len=${#text}
    local pad=$(( (34 - len) / 2 ))
    printf "\n${C}   ╭──────────────────────────────────────────╮${N}\n"
    printf "${C}   │${B}   "
    printf "%*s" $pad ""
    printf "${C}%s${N}" "$text"
    printf "%*s" $((34 - len - pad)) ""
    printf "   ${C}│${N}\n"
    printf "${C}   ╰${Y}C··································${C}╯${N}\n\n"
}

pacman_rem() {
    local label="$1"
    local i
    local frames=('C··········' '·C·········' '··C········' '···C·······' '····C······'
                  '·····C·····' '······C····' '·······C···' '········C··' '·········C·')
    for i in {0..9}; do
        printf "\r  ${Y}%s${N} ${R}%s${N} ${D}...${N}" "${frames[i]}" "$label"
        sleep 0.06
    done
    printf "\r  ${R}◆${N} %-28s ${R}removed${N}\n" "$label"
}

rem()  { pacman_rem "$1"; }
skip() { printf "  ${D}◆${N} %-28s ${D}not found${N}\n" "$1"; }

pacman_banner "backup-games — uninstall"

printf "   ${D}──${N} ${B}${C}Removing binary${N}\n"
if [ -f "$BIN" ]; then
    rm -f "$BIN"
    rem "Binary"
else
    skip "Binary"
fi

echo
printf "   ${D}──${N} ${B}${C}Removing config & data${N}\n"
if [ -d "$CONFIG_DIR" ]; then
    rm -rf "$CONFIG_DIR"
    rem "Config"
else
    skip "Config"
fi
if [ -d "$DATA_DIR" ]; then
    rm -rf "$DATA_DIR"
    rem "Data"
else
    skip "Data"
fi

echo
printf "   ${D}──${N} ${B}${C}Removing shell aliases${N}\n"
aliases_removed=0
for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
    [ -f "$rc" ] || continue
    if grep -q "alias $PROJECT=" "$rc" 2>/dev/null || grep -q "alias bckup-games=" "$rc" 2>/dev/null; then
        sed -i "/^alias $PROJECT=/d" "$rc"
        sed -i "/^alias bckup-games=/d" "$rc"
        aliases_removed=$((aliases_removed+1))
        rem "Aliases ($(basename "$rc"))"
    fi
done
fish="$HOME/.config/fish/config.fish"
if [ -f "$fish" ]; then
    if grep -q "alias $PROJECT " "$fish" 2>/dev/null || grep -q "alias bckup-games " "$fish" 2>/dev/null; then
        sed -i "/^alias $PROJECT /d" "$fish"
        sed -i "/^alias bckup-games /d" "$fish"
        aliases_removed=$((aliases_removed+1))
        rem "Aliases (config.fish)"
    fi
fi
[ "$aliases_removed" -eq 0 ] && skip "Aliases"

printf "\n  ${B}${G}◆  Uninstall complete${N}\n"
echo
