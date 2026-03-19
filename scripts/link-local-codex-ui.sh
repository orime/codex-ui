#!/bin/sh
set -eu

ROOT="$(CDPATH='' cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/codex-rs/target/debug/codex"
LINK_TARGET="${1:-$HOME/.n/bin/codex-ui}"

if [ ! -x "$BIN" ]; then
  echo "missing local debug binary: $BIN" >&2
  echo "run: cargo +stable build --manifest-path $ROOT/codex-rs/Cargo.toml -p codex-cli --bin codex" >&2
  exit 1
fi

mkdir -p "$(dirname "$LINK_TARGET")"
ln -sfn "$BIN" "$LINK_TARGET"
echo "linked $LINK_TARGET -> $BIN"
