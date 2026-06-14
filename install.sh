#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
INSTALL_PREFIX=${DEEPLENS_INSTALL_PREFIX:-"$HOME/.local"}
BIN_DIR="$INSTALL_PREFIX/bin"
APP_BIN="$ROOT_DIR/target/release/DeepLens"
APP_ICON="$ROOT_DIR/assets/DeepLens.icns"

if [ "${DEEPLENS_APP_DIR:-}" ]; then
    APP_DIR=$DEEPLENS_APP_DIR
elif [ -w /Applications ]; then
    APP_DIR=/Applications
else
    APP_DIR="$HOME/Applications"
fi

APP_BUNDLE="$APP_DIR/DeepLens.app"
APP_CONTENTS="$APP_BUNDLE/Contents"
APP_MACOS="$APP_CONTENTS/MacOS"
APP_RESOURCES="$APP_CONTENTS/Resources"

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

install_brew_package() {
    package=$1
    binary=$2

    if command_exists "$binary"; then
        printf '%s already installed\n' "$binary"
        return
    fi

    printf 'Installing %s with Homebrew...\n' "$package"
    brew install "$package"
}

if [ "$(uname -s)" != "Darwin" ]; then
    printf 'DeepLens is currently macOS-focused. This installer expects macOS.\n' >&2
    exit 1
fi

if ! command_exists brew; then
    printf 'Homebrew is required to install DeepLens dependencies.\n' >&2
    printf 'Install Homebrew from https://brew.sh, then run ./install.sh again.\n' >&2
    exit 1
fi

if ! command_exists cargo; then
    printf 'Rust/Cargo is required to build DeepLens from source.\n' >&2
    printf 'Install Rust from https://rustup.rs, then run ./install.sh again.\n' >&2
    exit 1
fi

install_brew_package ripgrep-all rga
install_brew_package fd fd
install_brew_package zoxide zoxide

if command_exists numbat; then
    printf 'numbat already installed\n'
else
    printf 'Installing numbat with Cargo...\n'
    cargo install numbat-cli
fi

printf 'Building DeepLens release binary...\n'
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

mkdir -p "$BIN_DIR"
install -m 755 "$APP_BIN" "$BIN_DIR/DeepLens"

mkdir -p "$APP_MACOS" "$APP_RESOURCES"
install -m 755 "$APP_BIN" "$APP_MACOS/DeepLens"
if [ -f "$APP_ICON" ]; then
    install -m 644 "$APP_ICON" "$APP_RESOURCES/DeepLens.icns"
fi

cat > "$APP_CONTENTS/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>DeepLens</string>
    <key>CFBundleExecutable</key>
    <string>DeepLens</string>
    <key>CFBundleIdentifier</key>
    <string>app.deeplens.DeepLens</string>
    <key>CFBundleIconFile</key>
    <string>DeepLens</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>DeepLens</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

printf 'APPL????' > "$APP_CONTENTS/PkgInfo"
touch "$APP_BUNDLE"

printf '\nRunning DeepLens doctor...\n'
"$BIN_DIR/DeepLens" --doctor

printf '\nDeepLens installed to %s/DeepLens\n' "$BIN_DIR"
printf 'DeepLens.app installed to %s\n' "$APP_BUNDLE"

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
        printf 'Add this to your shell profile if it is not already on PATH:\n'
        printf '  export PATH="%s:$PATH"\n' "$BIN_DIR"
        ;;
esac

printf '\nRun with:\n'
printf '  DeepLens\n'
printf '  open "%s"\n' "$APP_BUNDLE"
printf '  DeepLens --doctor\n'
