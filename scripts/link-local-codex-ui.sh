#!/bin/sh
set -eu

ROOT="$(CDPATH='' cd "$(dirname "$0")/.." && pwd)"
PROFILE="${CODEX_UI_PROFILE:-debug}"
case "$PROFILE" in
  debug | release) ;;
  *)
    echo "unsupported CODEX_UI_PROFILE: $PROFILE" >&2
    echo "expected: debug or release" >&2
    exit 1
    ;;
esac

BIN="${CODEX_UI_BINARY_PATH:-$ROOT/codex-rs/target/$PROFILE/codex}"
LINK_TARGET="${1:-$HOME/.n/bin/codex-ui}"

if [ ! -x "$BIN" ]; then
  echo "missing local $PROFILE binary: $BIN" >&2
  if [ "$PROFILE" = "release" ]; then
    echo "run: cargo +stable build --manifest-path $ROOT/codex-rs/Cargo.toml --release -p codex-cli --bin codex" >&2
  else
    echo "run: cargo +stable build --manifest-path $ROOT/codex-rs/Cargo.toml -p codex-cli --bin codex" >&2
  fi
  exit 1
fi

mkdir -p "$(dirname "$LINK_TARGET")"
rm -f "$LINK_TARGET"
cat >"$LINK_TARGET" <<EOF
#!/bin/sh
set -eu
exec "$BIN" -c 'tui.theme="opencode-matrix"' "\$@"
EOF
chmod 0755 "$LINK_TARGET"
echo "wrote $LINK_TARGET -> $BIN (tui.theme=opencode-matrix)"
