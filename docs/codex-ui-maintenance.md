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
10. Tag and push the `v0.125.0-ui.N` release.

## Release Flow

Current release assets are macOS-only:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

Release order:

1. Finish and verify locally.
2. Push `main`.
3. Tag, for example `v0.125.0-ui.1`.
4. Push the tag.
5. Let GitHub Actions build and publish the release.

Do not document a platform unless the workflow publishes that asset.
