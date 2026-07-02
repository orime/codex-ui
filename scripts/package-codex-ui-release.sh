#!/bin/sh

set -eu

target="${1:?usage: package-codex-ui-release.sh <target> [dist-dir]}"
dist_dir="${2:-dist}"
repo_root="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
binary_path="${CODEX_UI_BINARY_PATH:-$repo_root/codex-rs/target/$target/release/codex}"

if [ ! -x "$binary_path" ]; then
  fallback="$repo_root/codex-rs/target/release/codex"
  if [ -x "$fallback" ]; then
    binary_path="$fallback"
  else
    echo "Missing codex binary: $binary_path" >&2
    echo "Set CODEX_UI_BINARY_PATH or build the release binary first." >&2
    exit 1
  fi
fi

theme_path="$repo_root/themes/opencode-matrix.tmTheme"
if [ ! -f "$theme_path" ]; then
  dist_theme="$repo_root/dist/opencode-matrix.tmTheme"
  if [ -f "$dist_theme" ]; then
    theme_path="$dist_theme"
  else
    echo "Missing opencode-matrix.tmTheme: $theme_path" >&2
    echo "Keep themes/opencode-matrix.tmTheme in the repository." >&2
    exit 1
  fi
fi

stage_dir="$repo_root/$dist_dir/stage-$target"
archive="$repo_root/$dist_dir/codex-ui-$target.tar.gz"
checksum="$repo_root/$dist_dir/codex-ui-$target.sha256"

rm -rf "$stage_dir"
mkdir -p "$stage_dir" "$repo_root/$dist_dir"

cp "$binary_path" "$stage_dir/codex-ui-bin"
cp "$theme_path" "$stage_dir/opencode-matrix.tmTheme"

cat >"$stage_dir/codex-ui" <<'EOF'
#!/bin/sh
set -eu
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "$SCRIPT_DIR/codex-ui-bin" \
  -c 'tui.theme="opencode-matrix"' \
  -c 'notify=[]' \
  -c 'features.computer_use=false' \
  -c 'plugins.computer-use@openai-bundled.enabled=false' \
  -c 'mcp_servers.computer-use={command="/bin/false", args=[], enabled=false}' \
  "$@"
EOF

chmod 0755 "$stage_dir/codex-ui" "$stage_dir/codex-ui-bin"
tar -C "$stage_dir" -czf "$archive" codex-ui codex-ui-bin opencode-matrix.tmTheme
shasum -a 256 "$archive" >"$checksum"

printf '%s\n' "$archive"
printf '%s\n' "$checksum"
