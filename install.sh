#!/usr/bin/env bash
set -euo pipefail

PROJECT="backup-games"
REPO_URL="https://github.com/arvin0494/backup-games.git"
_TMPDIR=""

R=$'\033[31m' G=$'\033[32m' Y=$'\033[33m' C=$'\033[36m' M=$'\033[35m'
B=$'\033[1m' D=$'\033[2m' N=$'\033[0m'

header() {
    printf "\n${M}██████╗  ${B}██╗  ██╗███████╗██╗     ██████╗ ${N}${M}███████╗██████╗${N}\n"
    printf "${M}██╔════╝  ${B}██║  ██║██╔════╝██║     ██╔══██╗${N}${M}██╔════╝██╔══██╗${N}\n"
    printf "${M}██║  ███╗${B}███████║█████╗  ██║     ██████╔╝${N}${M}█████╗  ██████╔╝${N}\n"
    printf "${M}██║   ██║${B}██╔══██║██╔══╝  ██║     ██╔═══╝ ${N}${M}██╔══╝  ██╔══██╗${N}\n"
    printf "${M}╚██████╔╝${B}██║  ██║███████╗███████╗██║     ${N}${M}███████╗██║  ██║${N}\n"
    printf "${M} ╚═════╝ ${B}╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ${N}${M}╚══════╝╚═╝  ╚═╝${N}\n"
    printf "${D}                        ██╗     ██╗███╗   ██╗██╗   ██╗██╗  ██╗${N}\n"
    printf "${D}                        ██║     ██║████╗  ██║██║   ██║╚██╗██╔╝${N}\n"
    printf "${D}                        ██║     ██║██╔██╗ ██║██║   ██║ ╚███╔╝${N}\n"
    printf "${D}                        ██║     ██║██║╚██╗██║██║   ██║ ██╔██╗${N}\n"
    printf "${D}                        ███████╗██║██║ ╚████║╚██████╔╝██╔╝ ██╗${N}\n"
    printf "${D}                        ╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝${N}\n"
    printf "\n${D}    ░▒▓█████████████████████████████████████████████████████▓▒░${N}\n\n"
}

section() {
    local n="$1" title="$2"
    printf "\n${D}  ┌──────────────────────────────────────────────────────┐${N}\n"
    printf "${C}  │ [0x%02x] %-45s${N}\n" "$n" "$title"
    printf "${D}  └──────────────────────────────────────────────────────┘${N}\n"
}

info() { printf "${B}  ▸${N} %-24s ${C}%s${N}\n" "$1" "$2"; }

ok()   { printf "${B}  ▸${N} %-24s ${G}%s${N}\n" "$1" "$2"; }

bar() {
    local pct=$1
    local w=30 filled
    filled=$((pct * w / 100))
    printf "  ${D}[${N}"
    for ((i=0; i<w; i++)); do
        [ $i -lt $filled ] && printf "${G}█${N}" || printf "${D}░${N}"
    done
    printf "${D}]${N}  %3d%% %s\n" "$pct" "$2"
}

step()    { printf "  ${C}▶${N} %s\n" "$*"; }
warn()    { printf "  ${Y}⚠ %s${N}\n" "$*"; }
success() { printf "\n  ${G}${B}✔ %s${N}\n" "$*"; }
fail()    { printf "\n  ${R}${B}✘ %s${N}\n" "$*"; }

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

    step "Compiling (cargo build --release)..."
    cargo build --release 2>&1 | while IFS= read -r line; do
        [[ "$line" == "   Compiling "* ]] && bar 50 "${line#   }" && continue
        [[ "$line" == "    Finished"* ]] && bar 100 "Linking..."
    done
    bar 100 "Build complete"

    step "Installing binary..."
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
    if [ ! -f "$cfg_file" ]; then
        mkdir -p "$cfg_dir"
        cat > "$cfg_file" << 'EOF'
# backup-games configuration
sources=~/Games
dirsources=~/.local/share/honkers-railway-launcher
dest=/mnt/HDD4T/GAMES
min_size=1
EOF
        ok "Config" "$cfg_file"
    fi
}

cleanup() { [ -n "${_TMPDIR:-}" ] && rm -rf "$_TMPDIR"; }

main() {
    header

    printf "${D}  ╔══════════════════════════════════════════════════════╗${N}\n"
    printf "${D}  ║${N}  ${B}REMOTE DEPLOYMENT SEQUENCE${N}            ${D}rev 1.0       ║${N}\n"
    printf "${D}  ║${N}  PROTOCOL: ${Y}DOWNLOAD${N} ${D}→${N} ${C}VERIFY${N} ${D}→${N} ${M}INJECT${N} ${D}→${N} ${G}ARM${N}          ${D}║${N}\n"
    printf "${D}  ╚══════════════════════════════════════════════════════╝${N}\n\n"

    info "ROOT ACCESS" "CONFIRMED"
    info "USER" "$(whoami)"
    info "TARGET" "$HOME/.local/bin/$PROJECT"
    info "SOURCE" "$REPO_URL"

    section 1 "DOWNLOADING PAYLOADS FROM REMOTE"
    bar 14 "Fetching $PROJECT..."

    ensure_rust
    _TMPDIR="$(mktemp -d "/tmp/${PROJECT}-XXXXXX")"
    trap cleanup EXIT

    git clone --depth 1 "$REPO_URL" "$_TMPDIR/$PROJECT" 2>&1 | tail -1 || \
    git clone --depth 1 "${REPO_URL/https:\/\/github.com\//git@github.com:}" "$_TMPDIR/$PROJECT" 2>&1 | tail -1
    bar 100 "Repository cloned"

    section 2 "BUILDING BINARY"
    build_and_install "$_TMPDIR/$PROJECT"

    section 3 "CONFIGURING SYSTEM"
    shell_aliases
    create_config

    printf "\n${D}  ────────────────────────────────────────────────────────${N}\n"
    success "Deployment complete"
    step "Run ${B}${C}$PROJECT${N} or ${B}${C}bckup-games${N} ${D}--help${N}"
    echo
}

main "$@"
