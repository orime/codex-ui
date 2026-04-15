# codex-ui

[中文 README](./README.md)

![License](https://img.shields.io/github/license/orime/codex-ui)
![Release](https://img.shields.io/github/v/release/orime/codex-ui?display_name=tag)
![Workflow](https://img.shields.io/github/actions/workflow/status/orime/codex-ui/codex-ui-release.yml?label=release)

A UI-focused distribution of [openai/codex](https://github.com/openai/codex). The goal is simple: keep the official behavior intact while improving terminal visuals, Markdown rendering, and the overall `matrix`-style theme.

## What It Is

- Keeps the core `codex` behavior, auth flow, and usage model
- Improves TUI colors, Markdown rendering, tables, task lists, code blocks, and quote blocks
- Preserves the original `/theme` command for syntax highlighting and adds `/theme-ui` for full UI palette switching
- Includes 37 mapped opencode-style UI themes, with `matrix` as the default UI theme
- Binds the `codex-ui` launcher to `opencode-matrix.tmTheme` as the default syntax-highlighting companion for the `matrix` UI
- Does not overwrite an existing `codex` installation

Current upstream base:

- `rust-v0.118.0`

## Preview

![codex-ui preview](./docs/assets/preview.jpg)

This theme focuses on:

- cyan headings and structural emphasis
- yellow bold highlights
- warm inline emphasis
- darker code blocks and better block-level separation
- an overall look closer to `opencode matrix`

## Installation

### One-line installer

```sh
curl -fsSL https://raw.githubusercontent.com/orime/codex-ui/main/scripts/install-codex-ui.sh | sh
```

This installs the executables and `opencode-matrix.tmTheme`.
The installed `codex-ui` launcher also passes `-c 'tui.theme="opencode-matrix"'` by default.

Then run:

```sh
codex-ui
```

After launch:

- `/theme` still controls syntax highlighting
- `/theme-ui` switches the full UI palette
- the default UI theme is `matrix`

## Command Contract

This repository now treats the two launch paths as separate products:

- `codex-ui`: the stable user-facing command, reserved for the GitHub Release install
- `codex-ui-dev`: the local development command, reserved for binaries built from your working tree

Do not mix those responsibilities:

- use `codex-ui` for daily use and release verification
- use `codex-ui-dev` for local UI work and freshly compiled builds

The local linking helper now writes `codex-ui-dev` by default and no longer overwrites `codex-ui` unless you explicitly force it.

## Local Development Verification

During local UI work, do not point `codex-ui` at a workspace build. That causes two predictable failures:

- rebuilds appear to "do nothing" because you are accidentally launching an old binary
- deleting `target` breaks your day-to-day `codex-ui` command

Recommended loop:

```sh
cargo +stable build --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
./scripts/link-local-codex-ui.sh
codex-ui-dev --no-alt-screen
```

The helper script now writes a local `codex-ui-dev` launcher to:

- `~/.n/bin/codex-ui-dev`

That launcher binds:

- `codex-rs/target/debug/codex`
- `-c 'tui.theme="opencode-matrix"'`

So every rebuild is reflected immediately in your local `codex-ui-dev`, while the stable `codex-ui` command continues to point at the release install.

If you want the local development command to use the release build instead:

```sh
CODEX_UI_PROFILE=release ./scripts/link-local-codex-ui.sh
codex-ui-dev --no-alt-screen
```

If you really want to overwrite the stable `codex-ui` command, the script now refuses by default. You must opt in explicitly:

```sh
CODEX_UI_ALLOW_OVERWRITE_RELEASE=1 ./scripts/link-local-codex-ui.sh ~/.n/bin/codex-ui
```

That is intentionally not the normal workflow.

## How It Works

The default `codex-ui` experience intentionally binds two layers together:

- the built-in `matrix` UI palette
- the `opencode-matrix` syntax theme

The launcher is a thin wrapper that:

- launches `codex-ui-bin`
- automatically passes `-c 'tui.theme="opencode-matrix"'`

That means your existing:

- `~/.codex/auth.json`
- `~/.codex/config.toml`
- `~/.codex/sessions`

continue to work as-is.

This means `opencode-matrix` is not a random extra file. It is part of the default `codex-ui` visual contract on top of upstream `codex`.

By default, the UI palette uses the built-in `matrix` theme. Use `/theme-ui` to switch the full UI palette, and use `/theme` if you only want to change syntax highlighting.

## Release Status

The repository already includes a release workflow:

- [codex-ui-release.yml](./.github/workflows/codex-ui-release.yml)

It builds and publishes:

- `codex-ui-aarch64-apple-darwin.tar.gz`
- `codex-ui-x86_64-apple-darwin.tar.gz`
- `codex-ui-x86_64-unknown-linux-musl.tar.gz`
- matching `.sha256` files
- installer script `install-codex-ui.sh`

If you do not see any GitHub Releases yet, the usual reasons are:

1. no tag has been pushed
2. GitHub Actions is disabled for the repo
3. the workflow files were initially pushed with a token that lacked workflow-related permission

To trigger the first release:

```sh
cd /Users/orime/codex-ui
git tag v0.114.0-ui.1
git push origin v0.114.0-ui.1
```

## Local Build

```sh
git clone https://github.com/orime/codex-ui.git
cd codex-ui
cargo +stable build --manifest-path codex-rs/Cargo.toml --release --bin codex
```

Then package it with:

```sh
./scripts/package-codex-ui-release.sh aarch64-apple-darwin dist
```

## Search Keywords

Useful terms for discovery:

- OpenAI Codex
- Codex CLI
- Rust TUI
- terminal UI
- matrix theme
- markdown rendering
- codex theme
- opencode-inspired theme

## License

This repository is derived from OpenAI's open-source Codex project and remains under Apache-2.0.

- Upstream: [openai/codex](https://github.com/openai/codex)
- This repo preserves [LICENSE](./LICENSE) and [NOTICE](./NOTICE)
