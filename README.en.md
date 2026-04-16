# codex-ui

[中文 README](./README.md)

![License](https://img.shields.io/github/license/orime/codex-ui)
![Release](https://img.shields.io/github/v/release/orime/codex-ui?display_name=tag)
![Workflow](https://img.shields.io/github/actions/workflow/status/orime/codex-ui/codex-ui-release.yml?label=release)

A thin release wrapper around [openai/codex](https://github.com/openai/codex).

This branch has a narrow goal:

- stay aligned with the latest stable upstream
- avoid relying on a local `target` directory for day-to-day use
- build and publish releases remotely through GitHub Actions
- bundle the `opencode-matrix` syntax theme by default
- keep `codex-ui` and `codex-ui-dev` as separate command contracts

Current upstream alignment:

- upstream tag: `rust-v0.121.0`
- release date: `2026-04-15`

## Repository Boundary

This repository is intentionally kept as a thin wrapper:

- preserve official `codex` behavior, auth flow, command semantics, and runtime model
- inject `-c 'tui.theme="opencode-matrix"'` by default
- install `opencode-matrix.tmTheme` into `~/.codex/themes`
- never overwrite an existing `codex` command

That means:

- `codex-ui` is the release-facing command
- `codex-ui-bin` is the packaged upstream `codex` binary
- the release, install, and local-link workflows are all built around that wrapper

If deeper TUI UI fork work is needed later, it should be handled as a separate porting task instead of blindly merging old UI patches into a newer upstream base.

## Installation

### One-line installer

```sh
curl -fsSL https://raw.githubusercontent.com/orime/codex-ui/main/scripts/install-codex-ui.sh | sh
```

By default it will:

- download the latest Release package for the current platform
- install `codex-ui` and `codex-ui-bin` into `~/.local/bin`
- install `opencode-matrix.tmTheme` into `~/.codex/themes`
- leave any existing `codex` command untouched

Then run:

```sh
codex-ui
```

## Command Contract

This repository deliberately keeps two command paths separate:

- `codex-ui`: the stable end-user command, reserved for the GitHub Release install
- `codex-ui-dev`: the local development command, reserved for binaries built from your working tree

Do not mix them.

Recommended usage:

- use `codex-ui` for daily use, stable behavior, and release verification
- use `codex-ui-dev` for local code changes, UI iteration, and freshly built binaries

## Local Development

For local development, build first, then generate `codex-ui-dev`:

```sh
cargo +stable build --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
./scripts/link-local-codex-ui.sh
codex-ui-dev --no-alt-screen
```

The helper script writes to:

- `~/.n/bin/codex-ui-dev`

and injects:

- `-c 'tui.theme="opencode-matrix"'`

If you want the local development command to use a local release build instead:

```sh
CODEX_UI_PROFILE=release ./scripts/link-local-codex-ui.sh
codex-ui-dev --no-alt-screen
```

The script refuses to overwrite the stable `codex-ui` command by default. It only allows that when you opt in explicitly:

```sh
CODEX_UI_ALLOW_OVERWRITE_RELEASE=1 ./scripts/link-local-codex-ui.sh ~/.n/bin/codex-ui
```

## Release

The repository already includes a GitHub Release workflow:

- [codex-ui-release.yml](./.github/workflows/codex-ui-release.yml)

On tag push it will automatically:

- build `aarch64-apple-darwin`
- build `x86_64-apple-darwin`
- build `x86_64-unknown-linux-musl`
- generate packaged artifacts, checksums, and the installer script
- publish a GitHub Release

### Trigger a release

```sh
git tag v0.121.0-ui.1
git push origin main
git push origin v0.121.0-ui.1
```

After that, GitHub Actions handles the remote build. No local release build is required.

## Upgrade Locally to the Latest Release

```sh
export http_proxy=http://127.0.0.1:7890 https_proxy=http://127.0.0.1:7890
unset all_proxy ALL_PROXY
curl -fsSL https://raw.githubusercontent.com/orime/codex-ui/main/scripts/install-codex-ui.sh | sh
```

If your environment needs it, apply your usual `proxy` / `unproxy` switch before running the installer.

## Why This Structure

The old repository state mixed two very different concerns:

- upstream upgrades
- deep TUI UI fork work

Those do not move at the same speed. Keeping them tightly coupled causes predictable failures:

- huge conflict surfaces during upstream upgrades
- day-to-day commands depending on a local `target`
- deleting build artifacts breaking the stable command too

The current rule is:

- keep `codex-ui` as a releasable wrapper on top of the latest upstream
- port deeper UI changes later as separate, explicit work

That is the maintainable boundary.
