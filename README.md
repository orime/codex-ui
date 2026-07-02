# codex-ui

[English README](./README.en.md)

![License](https://img.shields.io/github/license/orime/codex-ui)
![Release](https://img.shields.io/github/v/release/orime/codex-ui?display_name=tag)
![Workflow](https://img.shields.io/github/actions/workflow/status/orime/codex-ui/codex-ui-release.yml?label=release)

基于 [openai/codex](https://github.com/openai/codex) 最新内核的深度 TUI UI fork。

这个仓库不是“只换个主题”的薄包装器。

它的目标是：

- 跟上官方最新稳定版内核
- 保留 `codex` 的核心能力、协议和认证链路
- 在高可见 TUI 页面持续落实 `codex-ui` 的视觉和交互语言
- 通过 GitHub Actions 远程编译并发布 Release

当前对齐版本：

- 上游 tag：`rust-v0.142.5`
- 发布时间：`2026-05-08`

## 给 AI 的升级入口

下次上游发布新版本时，可以直接把这句话交给 AI：

```text
进入这个仓库，按 docs/codex-ui-maintenance.md 的 Standard Upgrade Procedure，
把 codex-ui 升级到最新 openai/codex Rust 稳定 tag。不要只换主题，
必须保留上游 core/API/行为，并把 codex-ui 的高可见 TUI 消费层样式迁移到新结构上。
本地必须验证 codex-ui --version，远程必须等 GitHub Release
能通过 gh release view 查到完整 assets 后才算发布完成。
```

更完整的可复制提示词见：

- [docs/codex-ui-ai-upgrade-prompt.md](./docs/codex-ui-ai-upgrade-prompt.md)

## 仓库定位

`codex-ui` 的维护边界是：

- 内核层尽量贴近上游 `openai/codex`
- UI 层明确做 fork，不接受“只接主题底座、不接消费层”的半移植
- `codex-ui` 发布命令永远面向稳定可用版本

当前这条 `0.142.5` 线已经把一批高可见页面迁回 `codex-ui` 的设计语言，包括但不限于：

- onboarding / auth / trust directory
- update prompt / model migration
- selection list / resume picker / oss selection
- request-user-input / approval overlay / mcp elicitation
- history cell / hook cell / multi agents / plugins
- exec / diff / footer / feedback 等核心交互面板

这意味着：以后升级内核时，必须把“消费层 UI”当成主任务本身，而不是只恢复 `style.rs` 或主题文件就算完成。

## 安装

### 一键安装

```sh
curl -fsSL https://raw.githubusercontent.com/orime/codex-ui/main/scripts/install-codex-ui.sh | sh
```

默认会做这些事：

- 下载当前平台对应的 Release 包
- 安装 `codex-ui` 和 `codex-ui-bin` 到 `~/.local/bin`
- 安装 `opencode-matrix.tmTheme` 到 `~/.codex/themes`
- 不覆盖已有的 `codex`
- 使用内置的 `codex-ui` 深度定制 TUI，并默认注入 `opencode-matrix`

安装完成后直接运行：

```sh
codex-ui
```

## 本地开发

本地开发时，直接构建并运行 `codex` binary；正式本机使用只更新 `codex-ui-bin`，不要让 `codex-ui` 依赖工作区 `target` 目录：

```sh
cargo +stable build --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
install -m 755 codex-rs/target/debug/codex ~/.n/bin/codex-ui-bin
codex-ui --version
```

## Release

仓库里已经包含 GitHub Release 工作流：

- [codex-ui-release.yml](./.github/workflows/codex-ui-release.yml)

它会在 push tag 时自动：

- 构建 `aarch64-apple-darwin`
- 构建 `x86_64-apple-darwin`
- 生成打包产物、校验和、安装脚本
- 发布 GitHub Release

当前 release 只发布 macOS 资产。这不是遗漏，是刻意收口后的现状。README、installer、workflow 必须保持一致，不能再出现“文档说支持 Linux，Release 里却没有产物”的情况。

### 触发发布

```sh
git tag v0.142.5-ui.1
git push origin main
git push origin v0.142.5-ui.1
```

正式流程不是“改完就打 tag”。

正确顺序是：

1. 本地完成 UI 移植和 smoke check
2. 跑 `cargo test -p codex-tui`
3. 跑 `just fix -p codex-tui`
4. 跑 `just fmt`
5. 合并到 `main`
6. 打 `v0.142.5-ui.N` tag
7. push tag，等待 GitHub Actions 远程 release

先本地验，再远程发。不要反过来。

## 本地升级到最新 Release

```sh
export http_proxy=http://127.0.0.1:7890 https_proxy=http://127.0.0.1:7890
unset all_proxy ALL_PROXY
curl -fsSL https://raw.githubusercontent.com/orime/codex-ui/main/scripts/install-codex-ui.sh | sh
```

如果是内网环境或代理异常，按你的终端习惯先做 `proxy` / `unproxy` 再执行。

## 维护原则

这条仓库线要同时守住两件事：

- 上游内核尽可能新
- `codex-ui` 的 UI 定制不能退化回官方默认外观

真正踩过的坑有三个：

1. 只迁主题基础设施，没有把 `update_prompt`、`approval_overlay`、`history_cell` 这类消费层页面接回去，结果发布出来还是朴素默认 UI。
2. README、installer、workflow 三方口径不一致，导致用户按文档安装却拿不到对应产物。
3. 让 `codex-ui` 直接依赖本地 `target` 目录，导致清理构建缓存后正式命令失效。

以后升级时，按这个顺序做：

1. 先对齐上游版本
2. 再补 UI 消费层
3. 本地编译验证
4. 最后才发远程 release

维护清单见：

- [docs/codex-ui-maintenance.md](./docs/codex-ui-maintenance.md)
