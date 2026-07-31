#!/bin/sh
# Start kanban's local web UI.
#
#   curl -fsSL https://th2ch-g.github.io/kanban/serve.sh | sh
#
# Read it first if you like - piping a script into a shell deserves that much.
# What it does: use kanban if you already have it, otherwise fetch the released
# static binary for this machine, otherwise build from source with cargo. Then
# run `kanban serve` and tell you where to point a browser.
#
# Nothing is installed system-wide. The downloaded binary lands in a temporary
# directory unless KANBAN_BIN_DIR says otherwise.
set -eu

REPO="th2ch-g/kanban"
PORT="${KANBAN_PORT:-8787}"
BIN_DIR="${KANBAN_BIN_DIR:-${TMPDIR:-/tmp}/kanban-bin}"

say()  { printf '%s\n' "$*" >&2; }
die()  { say "error: $*"; exit 1; }

target() {
    os=$(uname -s)
    arch=$(uname -m)
    [ "$os" = "Linux" ] || die "prebuilt binaries are Linux-only; see the cargo fallback below"
    case "$arch" in
        x86_64|amd64) echo "x86_64-unknown-linux-musl" ;;
        aarch64|arm64) echo "aarch64-unknown-linux-musl" ;;
        *) die "no prebuilt binary for $arch" ;;
    esac
}

download() {
    asset="kanban-$(target)"
    url="https://github.com/$REPO/releases/latest/download/$asset"
    mkdir -p "$BIN_DIR"
    say "fetching $asset"
    if ! curl -fsSL "$url" -o "$BIN_DIR/kanban"; then
        return 1
    fi

    # Every release publishes a checksum, so a missing one means something is
    # wrong - a truncated release, or someone in a position to drop that single
    # request. Refuse rather than run an unverified binary.
    curl -fsSL "$url.sha256" -o "$BIN_DIR/kanban.sha256" \
        || die "no checksum published for $asset; refusing to run it unverified"
    expected=$(cut -d' ' -f1 < "$BIN_DIR/kanban.sha256")
    actual=$(sha256sum "$BIN_DIR/kanban" | cut -d' ' -f1)
    [ -n "$expected" ] || die "empty checksum for $asset"
    [ "$expected" = "$actual" ] || die "checksum mismatch for $asset"
    say "checksum ok"

    chmod +x "$BIN_DIR/kanban"
    echo "$BIN_DIR/kanban"
}

build_from_source() {
    command -v cargo >/dev/null 2>&1 || die "no release binary for this machine and no cargo to build one"
    say "building from source; this takes a few minutes"
    cargo install --git "https://github.com/$REPO.git" kanban --locked >&2
    command -v kanban
}

if command -v kanban >/dev/null 2>&1; then
    KANBAN=$(command -v kanban)
    say "using the kanban already on your PATH"
elif KANBAN=$(download 2>/dev/null); then
    :
else
    say "no release binary available; falling back to cargo"
    KANBAN=$(build_from_source)
fi

say ""
say "  open http://127.0.0.1:$PORT/"
say ""

exec "$KANBAN" serve --port "$PORT"
