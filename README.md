# codex-matrix-ui

`codex-matrix-ui` 是基于 [openai/codex](https://github.com/openai/codex) 的 UI 定制发行版。

目标很直接：

- 保持官方 `codex` 的核心行为、认证方式和使用习惯
- 单独提供一个平行命令 `codex-ui`
- 重点修改 TUI 视觉、Markdown 渲染和 `opencode matrix` 风格主题
- 不覆盖用户原本安装的 `codex`

这个仓库以 Apache-2.0 许可的上游 Codex 为基础维护，当前本地基线来自上游提交 `a3613035f32a45146297a74e058a8c70b91c56c2`。

## 适合谁

- 你已经在用官方 `codex`，但嫌终端 UI 太素
- 你想要接近 `opencode matrix` 的配色和 Markdown 层次
- 你希望保留 `~/.codex` 的登录态、配置和工作流

## 这个发行版改了什么

- TUI 公共配色和 surface 风格
- Markdown 标题、粗体、斜体、链接、引用、任务列表、表格、数学片段渲染
- 代码块和行内代码层次
- 选择弹窗、状态条、会话区域等视觉细节
- 附带 `opencode-matrix.tmTheme`

## 这个发行版没改什么

- 模型调用方式
- 登录和认证逻辑
- 命令语义
- 工作目录、sandbox、approval 等行为

换句话说，日常使用上可以把它理解成：

- `codex` 是官方原版
- `codex-ui` 是 UI 强化版

## 安装

### 一键安装

```sh
curl -fsSL https://raw.githubusercontent.com/orime/codex-matrix-ui/main/scripts/install-codex-matrix-ui.sh | sh
```

默认会做这些事：

- 下载当前平台对应的 `codex-ui` Release 包
- 安装 `codex-ui` 和 `codex-ui-bin` 到 `~/.local/bin`
- 安装 `opencode-matrix.tmTheme` 到 `~/.codex/themes`
- 不覆盖已有的 `codex`

安装完成后直接运行：

```sh
codex-ui
```

### 手动安装

从 GitHub Releases 下载你平台对应的压缩包，解压后会得到：

- `codex-ui`
- `codex-ui-bin`
- `opencode-matrix.tmTheme`

把：

- `codex-ui`
- `codex-ui-bin`

放进你的 `PATH` 目录，把：

- `opencode-matrix.tmTheme`

放到 `~/.codex/themes/` 即可。

## 工作方式

`codex-ui` 是一个很薄的包装命令。它会：

- 调起同目录下的 `codex-ui-bin`
- 自动附带 `-c 'tui.theme="opencode-matrix"'`

所以你不需要手动改 `~/.codex/config.toml` 才能用这套主题。

你现有的：

- `~/.codex/auth.json`
- `~/.codex/config.toml`
- `~/.codex/sessions`

都会继续复用。

## 平台支持

当前 Release workflow 默认构建这些目标：

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-musl`

如果后续需要，可以再补：

- `aarch64-unknown-linux-musl`

## 本地构建

```sh
git clone https://github.com/orime/codex-matrix-ui.git
cd codex-matrix-ui
cargo +stable build --manifest-path codex-rs/Cargo.toml --release --bin codex
```

构建完成后可以执行：

```sh
./scripts/package-codex-ui-release.sh aarch64-apple-darwin dist
```

它会把二进制重新封装成 Release 产物格式。

## 发布流程

1. 从上游 `openai/codex` 同步最新代码
2. 合入本仓库的 UI 改动
3. 打 tag，例如 `v0.114.0-ui.1`
4. 推送 tag
5. GitHub Actions 自动构建并发布对应平台的 Release 资产

Release 资产命名为：

- `codex-matrix-ui-aarch64-apple-darwin.tar.gz`
- `codex-matrix-ui-x86_64-apple-darwin.tar.gz`
- `codex-matrix-ui-x86_64-unknown-linux-musl.tar.gz`

## 维护建议

- 保持 `codex-ui` 作为独立命令，不要覆盖系统里的 `codex`
- 尽量把改动限制在 `codex-rs/tui` 和主题资产
- 版本号跟随上游，例如 `v0.114.0-ui.1`

## 许可与归属

本仓库基于 OpenAI 开源的 Codex 仓库修改而来，遵循 Apache-2.0。

- 上游项目：[openai/codex](https://github.com/openai/codex)
- 本仓库保留原始 `LICENSE` 与 `NOTICE`
