#!/usr/bin/env bash
set -euo pipefail

PROJECT="backup-games"
REPO_URL=""

ensure_rust() {
    if command -v cargo &>/dev/null; then
        return 0
    fi
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
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
    if [ -d "$PROJECT" ]; then
        echo "Project directory exists, pulling updates..."
        cd "$PROJECT"
        git pull
        cd ..
    else
        git clone --depth 1 "$REPO_URL" "$PROJECT" || \
        git clone --depth 1 "${REPO_URL/https:\/\/github.com\//git@github.com:}" "$PROJECT"
    fi
}

build_binary() {
    cd "$PROJECT"
    cargo build --release
    mkdir -p "$HOME/.local/bin"
    cp "target/release/$PROJECT" "$HOME/.local/bin/"
    echo "Installed to ~/.local/bin/$PROJECT"
    cd ..
}

shell_aliases() {
    local alias_cmd="alias $PROJECT='$HOME/.local/bin/$PROJECT'"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.config/fish/config.fish"; do
        if [ -f "$rc" ] && ! grep -q "alias $PROJECT" "$rc" 2>/dev/null; then
            echo "$alias_cmd" >> "$rc"
            echo "Added alias to $rc"
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
# source=~/Games
# dest=/mnt/HDD4T/GAMES
EOF
        echo "Created default config at $cfg_file"
    fi
}

main() {
    ensure_rust
    [ -n "$REPO_URL" ] && clone_repo
    build_binary
    shell_aliases
    create_config
    echo "Installation complete! Run '$PROJECT --help'"
}

main "$@"
