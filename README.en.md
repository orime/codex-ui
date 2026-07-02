# codex-ui

[中文 README](./README.md)

![License](https://img.shields.io/github/license/orime/codex-ui)
![Release](https://img.shields.io/github/v/release/orime/codex-ui?display_name=tag)
![Workflow](https://img.shields.io/github/actions/workflow/status/orime/codex-ui/codex-ui-release.yml?label=release)

A deep TUI UI fork built on top of the latest [openai/codex](https://github.com/openai/codex) core.

This repository is not a "theme-only" wrapper.

Its goals are:

- stay aligned with the latest stable upstream core
- preserve the official `codex` runtime, protocol, and auth behavior
- keep applying the `codex-ui` visual and interaction language across high-visibility TUI surfaces
- build and publish releases remotely through GitHub Actions

Current upstream alignment:

- upstream tag: `rust-v0.142.5`
- release date: `2026-05-08`

## AI Upgrade Entry Point

When a new upstream version is released, give the AI agent this prompt:

```text
进入这个仓库，按 docs/codex-ui-maintenance.md 的 Standard Upgrade Procedure，
把 codex-ui 升级到最新 openai/codex Rust 稳定 tag。不要只换主题，
必须保留上游 core/API/行为，并把 codex-ui 的高可见 TUI 消费层样式迁移到新结构上。
本地必须验证 codex-ui --version，远程必须等 GitHub Release
能通过 gh release view 查到完整 assets 后才算发布完成。
```

Full reusable prompt:

- [docs/codex-ui-ai-upgrade-prompt.md](./docs/codex-ui-ai-upgrade-prompt.md)

## Repository Positioning

The maintenance boundary for `codex-ui` is:

- stay close to upstream for the core engine
- intentionally fork the UI layer
- reject half-ports where only theme infrastructure is restored but visible screens are left behind
- keep `codex-ui` as the stable release command

The current `0.142.5` line already ports a broad set of high-visibility screens back into the `codex-ui` design language, including:

- onboarding / auth / trust directory
- update prompt / model migration
- selection list / resume picker / oss selection
- request-user-input / approval overlay / mcp elicitation
- history cell / hook cell / multi agents / plugins
- exec / diff / footer / feedback and other core panels

That is the rule going forward: upgrading upstream is not done when `style.rs` or the theme file builds. It is only done when the visible consumer surfaces are ported too.

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
- launch the built-in `codex-ui` fork with `opencode-matrix` enabled by default

Then run:

```sh
codex-ui
```

## Local Development

For local development, build and run the `codex` binary directly. For normal local use, update `codex-ui-bin`; do not make `codex-ui` depend on a workspace `target` directory:

```sh
cargo +stable build --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
install -m 755 codex-rs/target/debug/codex ~/.n/bin/codex-ui-bin
codex-ui --version
```

## Release

The repository already includes a GitHub Release workflow:

- [codex-ui-release.yml](./.github/workflows/codex-ui-release.yml)

On tag push it will automatically:

- build `aarch64-apple-darwin`
- build `x86_64-apple-darwin`
- generate packaged artifacts, checksums, and the installer script
- publish a GitHub Release

Release assets are currently macOS-only by design. README, installer, and workflow must stay aligned. We do not document Linux release assets unless the workflow is actually publishing them.

### Trigger a release

```sh
git tag v0.142.5-ui.1
git push origin main
git push origin v0.142.5-ui.1
```

The correct order is:

1. finish the upstream UI port locally
2. run `cargo test -p codex-tui`
3. run `just fix -p codex-tui`
4. run `just fmt`
5. merge to `main`
6. tag `v0.142.5-ui.N`
7. push the tag and let GitHub Actions publish the release

Local verification comes first. Remote release comes after that.

## Upgrade Locally to the Latest Release

```sh
export http_proxy=http://127.0.0.1:7890 https_proxy=http://127.0.0.1:7890
unset all_proxy ALL_PROXY
curl -fsSL https://raw.githubusercontent.com/orime/codex-ui/main/scripts/install-codex-ui.sh | sh
```

If your environment needs it, apply your usual `proxy` / `unproxy` switch before running the installer.

## Maintenance Rules

This repository has to protect two things at the same time:

- keep the upstream core current
- keep the `codex-ui` UI from collapsing back into the default Codex look

The real failure modes we already hit are:

1. only porting theme infrastructure and forgetting visible screens such as `update_prompt`, `approval_overlay`, and `history_cell`
2. letting README, installer, and workflow drift apart
3. making `codex-ui` depend on a local `target` directory, so cleaning build caches breaks the stable command

Future upgrades should always follow this order:

1. align the upstream version
2. port the visible UI surfaces
3. verify locally
4. only then publish a remote release

See the maintenance checklist here:

- [docs/codex-ui-maintenance.md](./docs/codex-ui-maintenance.md)
