#!/usr/bin/env bash
set -euo pipefail

PROJECT="backup-games"
REPO_URL="https://github.com/arvin0494/backup-games.git"

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

clone_repo() {
    git clone --depth 1 "$REPO_URL" "$PROJECT" || \
    git clone --depth 1 "${REPO_URL/https:\/\/github.com\//git@github.com:}" "$PROJECT"
}

build_binary() {
    cargo build --release
    mkdir -p "$HOME/.local/bin"
    cp "target/release/$PROJECT" "$HOME/.local/bin/"
    echo "Installed to ~/.local/bin/$PROJECT"
}

shell_aliases() {
    local bin="$HOME/.local/bin/$PROJECT"
    local alias1="alias $PROJECT='$bin'"
    local alias2="alias bckup-games='$bin'"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.config/fish/config.fish"; do
        if [ -f "$rc" ]; then
            ! grep -q "alias $PROJECT=" "$rc" 2>/dev/null && echo "$alias1" >> "$rc" && echo "Added alias $PROJECT to $rc"
            ! grep -q "alias bckup-games=" "$rc" 2>/dev/null && echo "$alias2" >> "$rc" && echo "Added alias bckup-games to $rc"
        fi
    done
}

create_config() {
    local cfg_dir="$HOME/.config/$PROJECT"
    local cfg_file="$cfg_dir/config"
    if [ ! -f "$cfg_file" ]; then
        mkdir -p "$cfg_dir"
        cat > "$cfg_file" << 'EOF'
# backup-games configuration
source=~/Games
dest=/mnt/HDD4T/GAMES
EOF
        echo "Created default config at $cfg_file"
    fi
}

main() {
    ensure_rust

    if [ -f "Cargo.toml" ]; then
        BUILD_DIR="$PWD"
    else
        clone_repo
        BUILD_DIR="$PWD/$PROJECT"
    fi

    cd "$BUILD_DIR"
    build_binary
    shell_aliases
    create_config
    echo "Installation complete! Run '$PROJECT --help'"
}

main "$@"
