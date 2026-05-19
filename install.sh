#!/usr/bin/env sh
set -eu

REPO="hu-qi/x-cli-rs"
VERSION="${XCLI_RS_VERSION:-latest}"
INSTALL_DIR="${XCLI_RS_INSTALL_DIR:-$HOME/.local/bin}"
TMP_DIR="${TMPDIR:-/tmp}/x-cli-rs-install-$$"
BINS="x chatgpt-image-cli google-cli baidu-cli nanobanana-cli xiaohongshu-cli"

say() {
  printf '%s\n' "$*" >&2
}

fail() {
  say "error: $*"
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

need_cmd uname
need_cmd mkdir
need_cmd chmod
need_cmd unzip

if command -v curl >/dev/null 2>&1; then
  DOWNLOADER="curl"
elif command -v wget >/dev/null 2>&1; then
  DOWNLOADER="wget"
else
  fail "missing required command: curl or wget"
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
      *) fail "unsupported Linux architecture: $ARCH" ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
      x86_64|amd64) TARGET="x86_64-apple-darwin" ;;
      *) fail "unsupported macOS architecture: $ARCH" ;;
    esac
    ;;
  MINGW*|MSYS*|CYGWIN*)
    TARGET="x86_64-pc-windows-msvc"
    ;;
  *)
    fail "unsupported operating system: $OS"
    ;;
esac

if [ "$VERSION" = "latest" ]; then
  BASE_URL="https://github.com/$REPO/releases/latest/download"
else
  BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
fi

ARCHIVE="x-cli-rs-$TARGET.zip"
CHECKSUM="$ARCHIVE.sha256"

mkdir -p "$TMP_DIR" "$INSTALL_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

download() {
  url="$1"
  out="$2"
  if [ "$DOWNLOADER" = "curl" ]; then
    curl -fsSL "$url" -o "$out"
  else
    wget -q "$url" -O "$out"
  fi
}

say "Installing x-cli-rs"
say "  repo:    $REPO"
say "  version: $VERSION"
say "  target:  $TARGET"
say "  dir:     $INSTALL_DIR"

say "Downloading $ARCHIVE"
download "$BASE_URL/$ARCHIVE" "$TMP_DIR/$ARCHIVE"

say "Downloading $CHECKSUM"
download "$BASE_URL/$CHECKSUM" "$TMP_DIR/$CHECKSUM"

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$TMP_DIR" && sha256sum -c "$CHECKSUM")
elif command -v shasum >/dev/null 2>&1; then
  expected="$(awk '{print $1}' "$TMP_DIR/$CHECKSUM")"
  actual="$(shasum -a 256 "$TMP_DIR/$ARCHIVE" | awk '{print $1}')"
  [ "$expected" = "$actual" ] || fail "checksum mismatch"
else
  fail "missing required command: sha256sum or shasum"
fi

unzip -q "$TMP_DIR/$ARCHIVE" -d "$TMP_DIR/bin"

for bin in $BINS; do
  src="$TMP_DIR/bin/$bin"
  if [ ! -f "$src" ] && [ -f "$TMP_DIR/bin/$bin.exe" ]; then
    src="$TMP_DIR/bin/$bin.exe"
  fi
  [ -f "$src" ] || fail "missing binary in archive: $bin"
  cp "$src" "$INSTALL_DIR/$(basename "$src")"
  chmod +x "$INSTALL_DIR/$(basename "$src")"
done

say "Installed:"
for bin in $BINS; do
  say "  $INSTALL_DIR/$bin"
done
say ""
say "Make sure $INSTALL_DIR is on your PATH."
