#!/usr/bin/env bash
set -euo pipefail

PROJECT="backup-games"
REPO_URL="https://github.com/arvin0494/backup-games.git"
_TMPDIR=""

ensure_rust() {
    if command -v cargo &>/dev/null; then
        return 0
    fi
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
        if command -v cargo &>/dev/null; then
            return 0
        fi
    fi
    if [ -f "$HOME/.cargo/bin/cargo" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
        return 0
    fi
    echo "Rust not found. Install via https://rustup.rs"
    return 1
}

build_and_install() {
    local src="$1"

    cd "$src"
    cargo build --release

    mkdir -p "$HOME/.local/bin"
    cp "target/release/$PROJECT" "$HOME/.local/bin/"
    echo "Installed to ~/.local/bin/$PROJECT"
}

shell_aliases() {
    local bin="$HOME/.local/bin/$PROJECT"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [ -f "$rc" ]; then
            ! grep -q "alias $PROJECT=" "$rc" 2>/dev/null && echo "alias $PROJECT='$bin'" >> "$rc" && echo "Added alias $PROJECT to $rc"
            ! grep -q "alias bckup-games=" "$rc" 2>/dev/null && echo "alias bckup-games='$bin'" >> "$rc" && echo "Added alias bckup-games to $rc"
        fi
    done
    local fish="$HOME/.config/fish/config.fish"
    if [ -f "$fish" ]; then
        ! grep -q "alias $PROJECT " "$fish" 2>/dev/null && echo "alias $PROJECT '$bin'" >> "$fish" && echo "Added alias $PROJECT to $fish"
        ! grep -q "alias bckup-games " "$fish" 2>/dev/null && echo "alias bckup-games '$bin'" >> "$fish" && echo "Added alias bckup-games to $fish"
    fi
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
        echo "Created default config at $cfg_file"
    fi
}

cleanup() {
    [ -n "${_TMPDIR:-}" ] && rm -rf "$_TMPDIR"
}

main() {
    ensure_rust

    _TMPDIR="$(mktemp -d "/tmp/${PROJECT}-XXXXXX")"
    trap cleanup EXIT

    git clone --depth 1 "$REPO_URL" "$_TMPDIR/$PROJECT" || \
    git clone --depth 1 "${REPO_URL/https:\/\/github.com\//git@github.com:}" "$_TMPDIR/$PROJECT"

    build_and_install "$_TMPDIR/$PROJECT"
    shell_aliases
    create_config
    echo "Installation complete! Run '$PROJECT --help'"
}

main "$@"
