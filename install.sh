#!/usr/bin/env bash
set -euo pipefail

PROJECT="backup-games"
REPO_URL="https://github.com/arvin0494/backup-games.git"
_TMPDIR=""

R=$'\033[31m' G=$'\033[32m' Y=$'\033[33m' C=$'\033[36m' B=$'\033[1m' D=$'\033[2m' N=$'\033[0m'

header() {
    printf "\n${C}   ╭──────────────────────────────────────────╮${N}\n"
    printf "${C}   │${B}          backup-games installer${N}           ${C}│${N}\n"
    printf "${C}   │${D}     Game Save Backup & Restore Tool${N}      ${C}│${N}\n"
    printf "${C}   ╰──────────────────────────────────────────╯${N}\n\n"
}

section() {
    local n="$1" title="$2"
    printf "   ${D}──${N} ${B}${C}%s${N} ${D}${title}${N}\n" "$n" "$title"
}

step()    { printf "  ${C}◇${N} %s\n" "$*"; }
ok()      { printf "  ${G}◆${N} %-28s ${G}%s${N}\n" "$1" "$2"; }
warn()    { printf "  ${Y}◇${N} %s\n" "$*"; }
info()    { printf "  ${D}◇${N} %-28s ${D}%s${N}\n" "$1" "$2"; }
success() { printf "\n  ${B}${G}◆  %s${N}\n" "$*"; }
fail()    { printf "\n  ${B}${R}◆  %s${N}\n" "$*"; }

spin() {
    local pid=$1 msg="$2" s
    s=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
    while kill -0 "$pid" 2>/dev/null; do
        for c in "${s[@]}"; do
            printf "\r  ${C}%s${N} %s" "$c" "$msg"
            sleep 0.08
        done
    done
    printf "\r  ${G}◆${N} %s\n" "$msg"
}

ensure_rust() {
    if command -v cargo &>/dev/null; then return 0; fi
    if [ -f "$HOME/.cargo/env" ]; then source "$HOME/.cargo/env"
        if command -v cargo &>/dev/null; then return 0; fi
    fi
    if [ -f "$HOME/.cargo/bin/cargo" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"; return 0
    fi
    fail "Rust not found — install via https://rustup.rs"
    return 1
}

build_and_install() {
    local src="$1"
    cd "$src"

    step "Compiling..."
    cargo build --release 2>&1 | while IFS= read -r line; do
        [[ "$line" == "   Compiling "* ]] && printf "\r  ${C}⠙${N} %s" "${line#   }"
        [[ "$line" == "    Finished"* ]] && printf "\r  ${G}◆${N} Build complete\n"
    done

    step "Installing..."
    mkdir -p "$HOME/.local/bin"
    cp "target/release/$PROJECT" "$HOME/.local/bin/"
    ok "Binary" "$HOME/.local/bin/$PROJECT"
}

shell_aliases() {
    local bin="$HOME/.local/bin/$PROJECT"
    local count=0
    for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        [ -f "$rc" ] || continue
        ! grep -q "alias $PROJECT=" "$rc" 2>/dev/null && echo "alias $PROJECT='$bin'" >> "$rc" && count=$((count+1))
        ! grep -q "alias bckup-games=" "$rc" 2>/dev/null && echo "alias bckup-games='$bin'" >> "$rc" && count=$((count+1))
    done
    local fish="$HOME/.config/fish/config.fish"
    if [ -f "$fish" ]; then
        ! grep -q "alias $PROJECT " "$fish" 2>/dev/null && echo "alias $PROJECT '$bin'" >> "$fish" && count=$((count+1))
        ! grep -q "alias bckup-games " "$fish" 2>/dev/null && echo "alias bckup-games '$bin'" >> "$fish" && count=$((count+1))
    fi
    [ "$count" -gt 0 ] && ok "Aliases" "added to shell rc files"
}

create_config() {
    local cfg_dir="$HOME/.config/$PROJECT"
    local cfg_file="$cfg_dir/config"
    if [ -f "$cfg_file" ]; then
        cp "$cfg_file" "${cfg_file}.bak"
        warn "Backed up existing config to ${cfg_file}.bak"
    fi
    mkdir -p "$cfg_dir"
    cat > "$cfg_file" << 'EOF'
# backup-games configuration
sources=~/Games
dirsources=~/Games/honkers-railway-launcher/HSR,~/.local/share/honkers
# exclude can be a dir name or absolute path (e.g. honkers-railway-launcher or ~/Games/honkers-railway-launcher)
exclude=~/Games/honkers-railway-launcher
dest=/mnt/HDD4T/GAMES
min_size=1
EOF
    ok "Config" "$cfg_file"
}

cleanup() { [ -n "${_TMPDIR:-}" ] && rm -rf "$_TMPDIR"; }

show_changelog() {
    local changelog="$1/CHANGELOG.md"
    if [ ! -f "$changelog" ]; then return; fi
    printf "\n   ${D}──${N} ${B}${C}What's new${N}\n"
    while IFS= read -r line; do
        case "$line" in
            "## v"*) printf "  ${B}${C}%s${N}\n" "${line### }" ;;
            "- "*)   printf "  ${D}%s${N}\n" "${line}" ;;
        esac
    done < "$changelog"
    echo
}

main() {
    if [ "${1:-}" = "--uninstall" ]; then
        exec "$(dirname "$0")/uninstall.sh"
    fi

    header

    info "User" "$(whoami)"
    info "Target" "$HOME/.local/bin/$PROJECT"

    echo
    section "1" "Fetching source"
    ensure_rust
    _TMPDIR="$(mktemp -d "/tmp/${PROJECT}-XXXXXX")"
    trap cleanup EXIT
    git clone --depth 1 "$REPO_URL" "$_TMPDIR/$PROJECT" 2>&1 | tail -1 || \
    git clone --depth 1 "${REPO_URL/https:\/\/github.com\//git@github.com:}" "$_TMPDIR/$PROJECT" 2>&1 | tail -1
    ok "Repository" "cloned"

    echo
    section "2" "Building binary"
    build_and_install "$_TMPDIR/$PROJECT"
    show_changelog "$_TMPDIR/$PROJECT"

    echo
    section "3" "Setting up"
    shell_aliases
    create_config

    echo
    success "Install complete!"
    step "Run ${B}${C}$PROJECT${N} or ${B}${C}bckup-games${N} ${D}--help${N}"
    echo
}

main "$@"
