# codex-ui

[English README](./README.en.md)

![License](https://img.shields.io/github/license/orime/codex-ui)
![Release](https://img.shields.io/github/v/release/orime/codex-ui?display_name=tag)
![Workflow](https://img.shields.io/github/actions/workflow/status/orime/codex-ui/codex-ui-release.yml?label=release)

基于 [openai/codex](https://github.com/openai/codex) 的薄包装发行版。

这条分支的目标很明确：

- 跟上官方最新稳定版
- 不依赖本地 `target` 目录做日常使用
- 通过 GitHub Actions 远程编译并发布 Release
- 默认附带 `opencode-matrix` 语法主题
- 明确拆开 `codex-ui` 和 `codex-ui-dev`

当前对齐版本：

- 上游 tag：`rust-v0.121.0`
- 发布时间：`2026-04-15`

## 项目边界

这个仓库当前刻意保持为“薄包装器”：

- 保留官方 `codex` 的核心行为、认证方式、命令语义和运行模型
- 默认注入 `-c 'tui.theme="opencode-matrix"'`
- 安装 `opencode-matrix.tmTheme` 到 `~/.codex/themes`
- 不覆盖用户现有的 `codex` 命令

这意味着：

- `codex-ui` 是一个发行命令
- `codex-ui-bin` 是打包后的上游 `codex` 二进制
- 仓库里的 release/安装/本地联调链路都是围绕这层薄包装展开

如果后续还要继续做更深的 TUI UI fork，应该作为单独移植任务处理，而不是把旧的 UI 改动直接无脑 merge 到新上游。

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

安装完成后直接运行：

```sh
codex-ui
```

## 命令约定

这个仓库固定区分两条链路：

- `codex-ui`：正式使用命令，只指向 GitHub Release 安装版
- `codex-ui-dev`：本地开发联调命令，只指向当前工作区编译出来的二进制

不要再把两者混用。

推荐约定：

- 日常使用、稳定体验、验证 release 包：用 `codex-ui`
- 本地改代码、联调、验证刚编译出的改动：用 `codex-ui-dev`

## 本地开发

本地开发时，先编译，再生成 `codex-ui-dev`：

```sh
cargo +stable build --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
./scripts/link-local-codex-ui.sh
codex-ui-dev --no-alt-screen
```

这个脚本默认写入：

- `~/.n/bin/codex-ui-dev`

并且默认注入：

- `-c 'tui.theme="opencode-matrix"'`

如果你想让本地开发命令改为使用本地 release 构建：

```sh
CODEX_UI_PROFILE=release ./scripts/link-local-codex-ui.sh
codex-ui-dev --no-alt-screen
```

脚本默认拒绝覆盖正式命令 `codex-ui`。只有显式带上下面这个开关时才会放行：

```sh
CODEX_UI_ALLOW_OVERWRITE_RELEASE=1 ./scripts/link-local-codex-ui.sh ~/.n/bin/codex-ui
```

## Release

仓库里已经包含 GitHub Release 工作流：

- [codex-ui-release.yml](./.github/workflows/codex-ui-release.yml)

它会在 push tag 时自动：

- 构建 `aarch64-apple-darwin`
- 构建 `x86_64-apple-darwin`
- 构建 `x86_64-unknown-linux-musl`
- 生成打包产物、校验和、安装脚本
- 发布 GitHub Release

### 触发发布

```sh
git tag v0.121.0-ui.1
git push origin main
git push origin v0.121.0-ui.1
```

之后由 GitHub Actions 远程编译即可，不需要本地 build release。

## 本地升级到最新 Release

```sh
export http_proxy=http://127.0.0.1:7890 https_proxy=http://127.0.0.1:7890
unset all_proxy ALL_PROXY
curl -fsSL https://raw.githubusercontent.com/orime/codex-ui/main/scripts/install-codex-ui.sh | sh
```

如果是内网环境或代理异常，按你的终端习惯先做 `proxy` / `unproxy` 再执行。

## 为什么这样收口

之前这个仓库里混着两件事：

- 上游升级
- 深度 TUI UI fork

它们的节奏完全不同。把两者绑死，会导致：

- 升上游版本时冲突面巨大
- 日常命令依赖本地 `target`
- 一删构建产物，正式命令也跟着坏

这次收口后的原则是：

- `codex-ui` 先稳定成最新上游的可发布包装版
- 更深的 UI 改动以后单独移植、单独验证

这才是可持续维护的结构。
