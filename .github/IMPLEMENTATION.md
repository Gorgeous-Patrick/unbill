# CI/CD Pipeline — Implementation

## File Structure

All CI/CD configuration lives under `.github/`:

```
.github/
├── DESIGN.md                        # What the pipeline does and why
├── IMPLEMENTATION.md                # This file
├── workflows/
│   ├── ci.yml                       # Continuous integration (format, lint, test)
│   ├── build.yml                    # Reusable: compile all release artifacts
│   ├── release.yml                  # Reusable: publish a release to all channels
│   ├── release-aur.yml              # Reusable: publish to AUR
│   ├── nightly.yml                  # Entry point: nightly builds
│   └── version.yml                  # Entry point: stable version releases
├── aur/
│   ├── unbill-cli-bin/PKGBUILD      # Stable CLI AUR package definition
│   ├── unbill-tui-bin/PKGBUILD      # Stable TUI AUR package definition
│   ├── unbill-bin/PKGBUILD          # Stable desktop AUR package definition
│   ├── unbill-cli-nightly-bin/PKGBUILD   # Nightly CLI AUR package definition
│   ├── unbill-tui-nightly-bin/PKGBUILD   # Nightly TUI AUR package definition
│   └── unbill-nightly-bin/PKGBUILD  # Nightly desktop AUR package definition
```

______________________________________________________________________

## Workflow Relationships

Entry-point workflows (`nightly.yml`, `version.yml`) call reusable workflows using the `workflow_call` trigger. Reusable workflows declare their inputs and secrets explicitly. Secrets propagate from the entry point through each level via `secrets: inherit`, and are also explicitly declared at each intermediate level to ensure propagation through nested calls.

The dependency chain is:

```
nightly.yml ──► build.yml
            └──► release.yml ──► release-aur.yml

version.yml ──► build.yml
            └──► release.yml ──► release-aur.yml
```

Build always completes before release. Within the release workflow, the GitHub release job and the AUR job run in parallel.

______________________________________________________________________

## CI Workflow (`ci.yml`)

Triggered on pushes to `main` and pull requests targeting `main`. Three jobs run in parallel: `fmt`, `clippy`, and `test`. All use `ubuntu-latest`. The `clippy` and `test` jobs use `Swatinem/rust-cache` keyed by default to avoid rebuilding unchanged dependencies.

The `RUSTFLAGS: "-D warnings"` environment variable is set at the workflow level so that both `clippy` and `test` jobs treat warnings as errors.

______________________________________________________________________

## Build Workflow (`build.yml`)

### CLI and TUI job (`build`)

Uses a matrix strategy with three entries: Linux x86_64, macOS aarch64, and Windows x86_64. Each entry specifies the Rust target triple, the runner OS, and a human-readable suffix used in artifact naming.

The Rust toolchain is installed via `dtolnay/rust-toolchain@stable` with the cross-compilation target specified. `Swatinem/rust-cache` is keyed by target triple so each platform has its own cache.

Both packages are built in a single `cargo build --release --target` invocation. After building, binaries are copied from `target/{target}/release/` to the workspace root using the canonical naming scheme. On Windows, the `.exe` extension is included. Artifacts are uploaded with names like `binaries-linux-x86_64`.

### Tauri job (`build-tauri`)

Uses the same three-platform matrix but without cross-compilation targets, since Tauri builds natively on each runner.

The Rust toolchain is installed with the `wasm32-unknown-unknown` target, which is required to compile the Leptos frontend. Git LFS is enabled on checkout because the Tauri icons are stored in LFS.

On Linux runners, the GTK and WebKit development libraries are installed via `apt-get` before building. These are: `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, and `librsvg2-dev`.

Trunk is installed via `jetli/trunk-action`. The Tauri CLI is installed globally via npm, which provides a prebuilt binary and is available on all runner types without requiring the pnpm workspace.

The `tauri-apps/tauri-action` action is invoked with `projectPath` set to `crates/unbill-tauri` and `tauriScript` set to `tauri` (the globally installed npm binary). The action's `beforeBuildCommand` in `tauri.conf.json` runs `trunk build` to compile the frontend before the Rust backend is compiled.

The action exposes an `artifactPaths` output containing a JSON array of all produced bundle paths. A shell script iterates this array, extracts each file extension, and renames the file to `unbill-{platform}.{ext}`. The `tr -d '\r'` step strips Windows carriage returns from the `jq` output. Renamed artifacts are uploaded as `binaries-tauri-{platform}`.

______________________________________________________________________

## Release Workflow (`release.yml`)

### GitHub release job

Downloads all artifacts from the current workflow run using `actions/download-artifact` with `merge-multiple: true`, which flattens all artifact containers into a single directory. The `softprops/action-gh-release` action creates or updates the GitHub release, attaches all files matching `unbill-*`, and sets the `make_latest` flag to the inverse of the `prerelease` input.

### AUR job

Conditionally runs when `aur_packages` is a non-empty JSON array. Calls `release-aur.yml` passing the tag and packages array. Secrets propagate via `secrets: inherit`. The `AUR_SSH_KEY` secret is also explicitly declared in the `workflow_call` secrets block to ensure it propagates through the nested call chain.

______________________________________________________________________

## Release AUR Workflow (`release-aur.yml`)

Uses a matrix strategy driven by `fromJSON(inputs.packages)` to run one job per package in parallel.

For each package, the version is derived from the tag in two steps: the `v` prefix is stripped (for version tags), and all hyphens are replaced with dots. The resulting string is written into the `pkgver` field of the PKGBUILD at `.github/aur/{package}/PKGBUILD` using `sed`. The `pkgrel` field is reset to 1.

The `jbouter/aur-releaser` action is then invoked. It creates a builder user, sets up the SSH key from `secrets.AUR_SSH_KEY`, adds AUR to known hosts using key types `rsa`, `ecdsa`, and `ed25519`, clones the package's AUR git repository, copies the updated PKGBUILD, runs `makepkg --printsrcinfo` to regenerate the `.SRCINFO` file, commits with the provided message, and pushes to AUR.

______________________________________________________________________

## AUR Package Definitions

### PKGBUILD structure

Each PKGBUILD is a minimal shell script following AUR conventions. The `pkgver` field is a placeholder (`0`) that gets overwritten by CI on each release. A `_tag` variable reconstructs the original hyphenated tag from `pkgver` by replacing dots with hyphens, since the GitHub release URL uses the original tag format.

Checksums are set to `SKIP` for all source files. This is intentional for a project that does not yet have a stable release process requiring verified checksums.

### CLI and TUI packages

The binary packages (`unbill-cli-bin`, `unbill-tui-bin` and their nightly counterparts) download the raw binary from the GitHub release, name it using the package name as the destination filename in the source declaration, and install it to `/usr/bin/` with executable permissions using `install -Dm755`.

### Desktop packages

The desktop packages (`unbill-bin`, `unbill-nightly-bin`) download the `.deb` bundle produced by the Tauri build. A Debian `.deb` file is an `ar` archive containing a `data.tar.gz` with the installed file tree. The `package()` function extracts `data.tar.gz` directly into `$pkgdir`, which installs all files (binary, desktop entry, icons) at their correct system paths.

The desktop packages declare runtime dependencies on the GTK and WebKit libraries required by Tauri: `cairo`, `desktop-file-utils`, `gdk-pixbuf2`, `glib2`, `gtk3`, `hicolor-icon-theme`, `libsoup`, `pango`, and `webkit2gtk-4.1`.

Nightly desktop and CLI/TUI packages declare `provides` and `conflicts` fields so that the nightly and stable variants of each tool are mutually exclusive — only one can be installed at a time.

______________________________________________________________________

## Version Management Implementation

`cargo release` is configured by `release.toml` at the workspace root with two settings: `publish = false` (skips crates.io publishing) and `shared-version = true` with `tag-name = "v{{version}}"` (produces a single workspace-level tag instead of per-crate tags).

The workspace version is defined once in `[workspace.package]` in `Cargo.toml`. All member crates inherit it via `version.workspace = true`. The `tauri.conf.json` version is kept in sync by `release-please-config.json`, which specifies it as an extra file with a JSONPath selector pointing to the top-level `version` field.

______________________________________________________________________

## Secrets Reference

| Secret name | Where configured | How used |
|---|---|---|
| `AUR_SSH_KEY` | Repository secrets | SSH private key for the AUR account; passed to `jbouter/aur-releaser` as `ssh_private_key` |
| `GITHUB_TOKEN` | Automatically provided | Used by `softprops/action-gh-release` to create GitHub releases |
