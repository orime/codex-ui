# codex-ui Maintenance

## Positioning

`codex-ui` is a deep TUI UI fork continuously ported on top of the latest stable `openai/codex` core.

The maintenance goal has two parts:

- keep the upstream core current
- keep the `codex-ui` visual and interaction language from regressing to the official default UI

Doing only the first part is not a complete upgrade.

## Command Contract

The repository must keep three command contracts stable:

- `codex`: the official command; this repository does not overwrite it
- `codex-ui`: the stable release command from GitHub Release, unless the user explicitly opts in to overwrite it
- `codex-ui-dev`: the local development command pointing at the current workspace build

Do not bind the stable `codex-ui` command to a local `target` directory by default.

The stable `codex-ui` launcher executes the sibling `codex-ui-bin` from its installation
directory. If `codex-ui --version` still reports an older version after a successful local
build, check the resolved launcher first:

```sh
type -a codex-ui
codex-ui --version
codex-ui-dev --version
```

Updating `codex-ui-dev` only refreshes the development launcher. To refresh the stable
local command before the GitHub Release is installed, replace the installed `codex-ui-bin`
with the release binary while preserving the launcher:

```sh
install -m 755 codex-rs/target/release/codex ~/.n/bin/codex-ui-bin
install -m 755 codex-rs/target/release/codex ~/.local/bin/codex-ui-bin
codex-ui --version
```

## UI Porting Rules

When upgrading upstream, do not stop after these surface-level changes:

- restoring `opencode-matrix.tmTheme`
- adjusting `style.rs`
- injecting `tui.theme="opencode-matrix"` into the launcher

Those are only the foundation. The visible consumer screens are the fragile part.

Review high-visibility screens first:

- onboarding / auth / trust directory
- update prompt / model migration
- selection list / list selection popup / resume picker / oss selection
- request user input / approval overlay / mcp server elicitation
- footer / feedback / pending approvals / app link view
- history cell / hook cell / plugins / multi agents
- exec / diff / status card

## Style Rule

High-visibility UI should prefer semantic `opencode_*` helpers from `codex-rs/tui/src/style.rs`.

Avoid letting scattered raw color helpers such as `.cyan()`, `.green()`, `.red()`, and `.magenta()` become the dominant styling layer again.

## Upgrade Checklist

1. Align with the upstream stable release tag.
2. Port theme infrastructure and visible UI consumers.
3. Fix README, installer, workflow, and release wording together.
4. Link `codex-ui-dev` and smoke test locally.
5. Run `cargo check -p codex-tui`.
6. Run `cargo test -p codex-tui`.
7. Run `just fix -p codex-tui`.
8. Run `just fmt`.
9. Merge to `main`.
10. Tag and push the `v<upstream-version>-ui.N` release.

## Standard Upgrade Procedure

Use this procedure for every future upstream release, for example `rust-v0.131.0`.

### 1. Confirm the target upstream tag

Do not guess the newest release. Confirm that the upstream Rust tag exists first:

```sh
git ls-remote --tags https://github.com/openai/codex.git 'refs/tags/rust-v0.131.0'
```

If network access is slow or blocked, use the user's proxy rule and retry with proxy enabled.

### 2. Create an upgrade branch

Start from the current `codex-ui` integration branch or `main`, depending on what is already
landed:

```sh
git switch main
git pull --ff-only
git switch -c upgrade/rust-v0.131.0-ui
```

If local work is dirty, inspect it first and do not reset or discard user changes.

### 3. Bring in the upstream core

Fetch the upstream source and align the Rust core with the target tag. The goal is not to build a
plain official Codex binary. The goal is:

- keep upstream core/API/behavior from `rust-v0.131.0`
- reapply the `codex-ui` visual and interaction language on top of it

Use a temporary upstream checkout/worktree when needed:

```sh
git fetch https://github.com/openai/codex.git rust-v0.131.0
```

Resolve conflicts in favor of upstream core behavior unless the change is explicitly part of the
`codex-ui` UI layer.

### 4. Port the UI layer intentionally

Do not stop after copying theme files or setting `tui.theme="opencode-matrix"`.

The required porting surface includes:

- `codex-rs/tui/src/style.rs` semantic `opencode_*` helpers
- theme files under `themes/`
- onboarding / auth / trust directory
- update prompt / model migration
- selection list / list selection popup / resume picker / oss selection
- request user input / approval overlay / mcp server elicitation
- footer / feedback / pending approvals / app link view
- history cell / hook cell / plugins / multi agents
- exec / diff / status card / markdown rendering
- release wrapper, installer, README, workflow wording

When upstream changed a screen, keep the new upstream behavior and port the `codex-ui` styling onto
the new structure. Do not replace a new upstream screen with an older fork copy unless the behavior
is verified to be equivalent.

### 5. Verify locally before publishing

Run the focused Rust checks from the repository root:

```sh
cargo fmt --manifest-path codex-rs/Cargo.toml --all
cargo check --manifest-path codex-rs/Cargo.toml -p codex-tui
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui
just fix -p codex-tui
```

Then build the local release binary:

```sh
cargo build --manifest-path codex-rs/Cargo.toml --release --bin codex
codex-rs/target/release/codex --version
```

The version must report the target upstream version, for example `codex-cli 0.131.0`.

### 6. Refresh local commands correctly

Refresh the development launcher first:

```sh
CODEX_UI_PROFILE=release ./scripts/link-local-codex-ui.sh
codex-ui-dev --version
```

If the user wants the normal `codex-ui` command to run the new local build before a GitHub Release
installer is available, update the installed `codex-ui-bin` while preserving the wrapper:

```sh
install -m 755 codex-rs/target/release/codex ~/.n/bin/codex-ui-bin
install -m 755 codex-rs/target/release/codex ~/.local/bin/codex-ui-bin
codex-ui --version
```

Always diagnose command confusion with:

```sh
type -a codex-ui
type -a codex-ui-dev
codex-ui --version
codex-ui-dev --version
```

### 7. Package and smoke test the local artifact

Build the local macOS package for the current machine:

```sh
CODEX_UI_BINARY_PATH=codex-rs/target/release/codex ./scripts/package-codex-ui-release.sh aarch64-apple-darwin dist
tmp_dir="$(mktemp -d)"
tar -xzf dist/codex-ui-aarch64-apple-darwin.tar.gz -C "$tmp_dir"
"$tmp_dir/codex-ui-bin" --version
grep -F 'tui.theme="opencode-matrix"' "$tmp_dir/codex-ui"
```

Use `x86_64-apple-darwin` instead on Intel macOS.

### 8. Push and publish through GitHub Actions

Commit and push the upgrade branch:

```sh
git push origin upgrade/rust-v0.131.0-ui
```

After the branch is merged or intentionally chosen as the release source, tag the release:

```sh
git tag v0.131.0-ui.1
git push origin v0.131.0-ui.1
```

Then monitor the remote build:

```sh
gh run list --repo orime/codex-ui --workflow codex-ui-release.yml --limit 5
gh run view <run-id> --repo orime/codex-ui --json status,conclusion,url,jobs
gh run watch <run-id> --repo orime/codex-ui --exit-status
```

If a tag has already been pushed, do not rewrite or delete it by default. Fix forward with
`v0.131.0-ui.2`.

### 9. Confirm the GitHub Release

The release is not complete until the GitHub Release exists and has all expected assets:

```sh
gh release view v0.131.0-ui.1 --repo orime/codex-ui --json tagName,url,assets,publishedAt
```

Expected macOS assets:

- `codex-ui-aarch64-apple-darwin.tar.gz`
- `codex-ui-aarch64-apple-darwin.sha256`
- `codex-ui-x86_64-apple-darwin.tar.gz`
- `codex-ui-x86_64-apple-darwin.sha256`
- `install-codex-ui.sh`

If the workflow is still building, say that clearly. Do not claim the remote release is published
until `gh release view` can see it.

## Lessons From The 0.130.0 Port

What went well:

- the final binary is based on upstream `rust-v0.130.0`, not a plain rebuild of the old fork
- high-visibility TUI consumer screens were ported, not just the theme base
- local focused validation was run: `cargo check -p codex-tui`, `cargo test -p codex-tui`,
  `just fix -p codex-tui`, and release build
- the release workflow now builds macOS assets remotely
- command contracts are documented so `codex`, `codex-ui`, and `codex-ui-dev` stay separate

What did not go well:

- the first local handoff refreshed `codex-ui-dev` but left the stable `codex-ui-bin` at `0.125.0`
- the tag `v0.130.0-ui.1` was pushed before the later documentation fix, so the documentation fix
  is on the branch but not inside that tag
- the remote release was described too early; the GitHub Release is only real after the Actions run
  finishes and `gh release view` returns assets

For future upgrades, do the command-path verification before telling the user that the local
`codex-ui` command is upgraded.

## Release Flow

Current release assets are macOS-only:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

Release order:

1. Finish and verify locally.
2. Push `main`.
3. Tag, for example `v0.130.0-ui.1`.
4. Push the tag.
5. Let GitHub Actions build and publish the release.

Do not document a platform unless the workflow publishes that asset.

## GitHub Actions Policy

`codex-ui` is a downstream UI fork, not the upstream OpenAI monorepo. Do not automatically run
upstream workflows that depend on OpenAI-only runner groups, expensive Bazel/V8 infrastructure, or
README assumptions that conflict with this fork's bilingual docs.

Keep these automatic:

- `codex-ui-release` on release tags
- lightweight checks that work on public GitHub-hosted runners and do not stage upstream release
  artifacts

Keep these manual-only unless the fork gets equivalent infrastructure:

- `sdk`
- `Bazel`
- `v8-canary`
- `rust-ci-full`
- upstream npm package staging in `ci`

If a future upstream sync reintroduces automatic triggers for those workflows, remove the automatic
`push` / `pull_request` triggers again and leave `workflow_dispatch` available for explicit runs.
