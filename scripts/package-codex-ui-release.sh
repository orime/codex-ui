#!/bin/sh

set -eu

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <target-triple> <output-dir>" >&2
  exit 1
fi

TARGET="$1"
OUTPUT_DIR="$2"
REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BINARY_PATH="${CODEX_UI_BINARY_PATH:-$REPO_ROOT/codex-rs/target/$TARGET/release/codex}"
STAGE_DIR="$OUTPUT_DIR/stage-$TARGET"
ARCHIVE_PATH="$OUTPUT_DIR/codex-matrix-ui-$TARGET.tar.gz"
CHECKSUM_PATH="$OUTPUT_DIR/codex-matrix-ui-$TARGET.sha256"

if [ ! -x "$BINARY_PATH" ]; then
  echo "Built codex binary not found: $BINARY_PATH" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"

cp "$BINARY_PATH" "$STAGE_DIR/codex-ui-bin"
cp "$REPO_ROOT/themes/opencode-matrix.tmTheme" "$STAGE_DIR/opencode-matrix.tmTheme"

cat >"$STAGE_DIR/codex-ui" <<'EOF'
#!/bin/sh
set -eu
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "$SCRIPT_DIR/codex-ui-bin" -c 'tui.theme="opencode-matrix"' "$@"
EOF

chmod 0755 "$STAGE_DIR/codex-ui"
chmod 0755 "$STAGE_DIR/codex-ui-bin"

tar -czf "$ARCHIVE_PATH" -C "$STAGE_DIR" codex-ui codex-ui-bin opencode-matrix.tmTheme

if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$ARCHIVE_PATH" >"$CHECKSUM_PATH"
elif command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$ARCHIVE_PATH" >"$CHECKSUM_PATH"
else
  echo "Neither shasum nor sha256sum is available." >&2
  exit 1
fi

echo "Packaged:"
echo "  $ARCHIVE_PATH"
echo "  $CHECKSUM_PATH"
