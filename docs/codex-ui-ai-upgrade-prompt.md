# Codex-UI AI Upgrade Prompt

Use this prompt when asking an AI agent to upgrade `codex-ui` after a new upstream Codex release.

## Short Prompt

```text
进入这个仓库，按 docs/codex-ui-maintenance.md 的 Standard Upgrade Procedure，
把 codex-ui 升级到最新 openai/codex Rust 稳定 tag。不要只换主题，
必须保留上游 core/API/行为，并把 codex-ui 的高可见 TUI 消费层样式迁移到新结构上。
本地必须验证 codex-ui-dev --version 和 codex-ui --version，远程必须等 GitHub Release
能通过 gh release view 查到完整 assets 后才算发布完成。
```

## Full Prompt

```text
请在当前 codex-ui 仓库执行一次标准升级。

目标：
- 先确认 openai/codex 最新 Rust 稳定 tag，例如 rust-v0.131.0，不要靠猜。
- 基于该上游 tag 升级 codex-ui 的 core/API/行为。
- 不要做裸官方编译，也不要只恢复 theme/style.rs。
- 按 codex-ui 既有设计理念迁移高可见 TUI 消费层，包括 onboarding、approval、
  update prompt、selection/resume picker、history cell、footer/status、exec/diff、
  markdown/render、plugins/multi agents 等。
- 尽量让 GitHub Actions 负责远程 release 构建，本地也要编译一个 release binary。

必须遵守：
- 先阅读 docs/codex-ui-maintenance.md，按 Standard Upgrade Procedure 执行。
- 遇到网络拉取慢或失败时按项目规则使用 proxy 重试。
- 不要 reset/hard checkout，不要丢弃用户已有改动。
- 稳定命令 codex-ui、开发命令 codex-ui-dev、官方命令 codex 三者不要混用。
- codex-ui wrapper 实际执行安装目录里的 codex-ui-bin；只更新 codex-ui-dev 不代表
  codex-ui 已升级。
- 如果 release tag 已经推过，不要默认改写远程 tag；用 vX.Y.Z-ui.2 修正。

本地验收：
- cargo fmt/check/test/fix 按 docs/codex-ui-maintenance.md 执行。
- cargo release build 成功，二进制 --version 是目标版本。
- codex-ui-dev --version 是目标版本。
- codex-ui --version 也是目标版本。
- 本地 package 脚本产物能解包，launcher 包含 tui.theme="opencode-matrix"。

远程验收：
- push 分支和 release tag。
- GitHub Actions release workflow 成功。
- gh release view vX.Y.Z-ui.N 能看到完整 assets：
  - codex-ui-aarch64-apple-darwin.tar.gz
  - codex-ui-aarch64-apple-darwin.sha256
  - codex-ui-x86_64-apple-darwin.tar.gz
  - codex-ui-x86_64-apple-darwin.sha256
  - install-codex-ui.sh

最后用中文总结：
- 升级到的上游版本
- 本地 codex-ui/codex-ui-dev 版本
- 本地包路径和 sha256
- 远程分支、commit、tag、Actions URL、Release URL
- 没完成的事项必须明确说，不能把还在跑的远程 release 说成已完成。
```

## Acceptance Rule

An upgrade is not complete until both statements are true:

- local `codex-ui --version` reports the target version
- `gh release view vX.Y.Z-ui.N --repo orime/codex-ui` returns the expected assets
