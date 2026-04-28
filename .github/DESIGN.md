# CI/CD Pipeline — Design

## Purpose

This document describes the design of the CI/CD pipeline for the unbill project. It covers what each workflow does, why it exists, and how the workflows relate to each other.

## Principles

- **Separation of concerns.** Building, releasing to GitHub, and distributing to package managers are independent concerns. Each is handled by a dedicated reusable workflow.
- **Reusability.** Entry-point workflows (nightly, version) are thin orchestrators that call reusable workflows. Logic lives in the reusable workflows, not the entry points.
- **Consistent artifact naming.** All release artifacts follow the pattern `unbill-{component}-{platform}` or `unbill-{platform}.{ext}` so that downstream consumers (package managers, release jobs) can predict filenames without inspecting the build output.
- **Two release tracks.** Nightly builds provide continuous access to the latest development state. Version releases are stable, manually triggered via `cargo release`.

______________________________________________________________________

## Workflows

### CI (`ci.yml`)

The continuous integration workflow. It runs on every push to `main` and on every pull request targeting `main`. It ensures that the codebase is always in a passing state before changes are merged.

It runs three independent checks in parallel:

**Format check.** Verifies that all Rust source files are formatted according to the project's `rustfmt` configuration. Any deviation fails the check.

**Clippy lint.** Runs the Rust linter across the core library, CLI, and TUI packages. Warnings are treated as errors. The Tauri package is excluded because it requires GTK system libraries that are unavailable on standard CI runners.

**Test suite.** Runs the full test suite for the core library and CLI packages.

______________________________________________________________________

### Build (`build.yml`)

A reusable workflow that compiles all release artifacts. It is called by both the nightly and version release entry points.

It runs two independent job groups in parallel:

**CLI and TUI binaries.** Builds the `unbill-cli` and `unbill-tui` packages as native release binaries for three platforms: Linux x86_64, macOS aarch64, and Windows x86_64. Each platform builds in parallel. Binaries are renamed to the canonical `unbill-{component}-{platform}` naming scheme before upload. On Windows, the `.exe` extension is preserved.

**Tauri desktop app.** Builds the Tauri desktop application for the same three platforms in parallel. The Leptos frontend is compiled to WebAssembly using Trunk as part of the build. On Linux, the required GTK and WebKit system libraries are installed before building. The Tauri action produces all available bundle formats for each platform (AppImage, deb, rpm on Linux; dmg on macOS; msi and nsis on Windows). All produced bundles are renamed to the canonical `unbill-{platform}.{ext}` naming scheme before upload.

All artifacts are uploaded to the GitHub Actions artifact store under names beginning with `binaries-`, grouped by platform and component. They are consumed by the release workflow in the same pipeline run.

______________________________________________________________________

### Release (`release.yml`)

A reusable workflow that publishes a release given a set of pre-built artifacts. It is called by both the nightly and version release entry points, which pass different parameters to control which distribution channels are targeted.

**Inputs:**

- `tag` — the git tag name to associate with the release
- `prerelease` — whether to mark the release as a pre-release on GitHub
- `body` — the release description text
- `aur_packages` — a JSON array of AUR package names to publish; an empty array skips AUR distribution entirely

**Secrets:**

- `AUR_SSH_KEY` — the SSH private key used to authenticate with the AUR git server

**GitHub release job.** Downloads all artifacts from the current pipeline run, merges them into a single directory, and creates or updates a GitHub release with the given tag. All files matching `unbill-*` are attached. For version releases, the release is marked as the latest. For nightly releases, it is marked as a pre-release.

**AUR release job.** Runs in parallel with the GitHub release job, conditionally on the `aur_packages` input being populated. Delegates to `release-aur.yml`.

______________________________________________________________________

### Release AUR (`release-aur.yml`)

A reusable workflow that publishes one or more AUR packages. It accepts a JSON array of package names and runs each as a parallel matrix job.

For each package, it updates the `pkgver` field in the corresponding PKGBUILD (stored at `.github/aur/{package}/PKGBUILD`) to the version derived from the release tag. Hyphens in the tag are converted to dots because the AUR `pkgver` field forbids hyphens. The `pkgrel` field is reset to 1. The `jbouter/aur-releaser` action then handles cloning the AUR repository, generating the `.SRCINFO` file, committing, and pushing.

The PKGBUILD files live inside the main repository rather than in separate AUR repositories, so version tracking and package definitions stay in sync with the source.

______________________________________________________________________

### Nightly (`nightly.yml`)

An entry-point workflow that runs automatically at midnight UTC every day, and can also be triggered manually via workflow dispatch. It produces a nightly pre-release from the current state of `main`.

It runs three sequential stages:

**Prepare.** Generates a timestamp-based tag in the format `nightly-YYYYMMDD-HHMMSS`.

**Build.** Calls `build.yml` to compile all artifacts.

**Release.** Calls `release.yml` with the generated tag, marked as a pre-release, with all six nightly AUR packages targeted: `unbill-cli-nightly-bin`, `unbill-tui-nightly-bin`, and `unbill-nightly-bin`.

______________________________________________________________________

### Version Release (`version.yml`)

An entry-point workflow that triggers whenever a git tag beginning with `v` is pushed. This is the stable release pipeline. Tags are created using `cargo release`, which bumps the version in `Cargo.toml` and `tauri.conf.json`, commits, and pushes the tag.

It runs two sequential stages:

**Build.** Calls `build.yml` to compile all artifacts.

**Release.** Calls `release.yml` with the pushed tag, marked as a stable release (not a pre-release, and marked as the latest), targeting the three stable AUR packages: `unbill-cli-bin`, `unbill-tui-bin`, and `unbill-bin`.

______________________________________________________________________

## AUR Packages

Six AUR packages are maintained, split between stable and nightly tracks.

**Stable packages** (`unbill-cli-bin`, `unbill-tui-bin`, `unbill-bin`) are updated on every version release. They provide the stable CLI binary, TUI binary, and desktop application respectively.

**Nightly packages** (`unbill-cli-nightly-bin`, `unbill-tui-nightly-bin`, `unbill-nightly-bin`) mirror the stable packages but are updated on every nightly release. Nightly packages declare conflicts with their stable counterparts so only one track can be installed at a time.

The CLI and TUI packages download a raw binary and install it directly. The desktop package downloads the `.deb` bundle produced by Tauri, extracts its contents using the standard Debian archive format, and installs them into the package directory. The desktop package declares all GTK and WebKit runtime dependencies required by Tauri applications.

All PKGBUILDs use a `_tag` variable to reconstruct the original hyphenated tag from the dot-separated `pkgver`, because the download URL on GitHub uses the original tag format while `pkgver` requires dots.

______________________________________________________________________

## Artifact Naming Convention

All release artifacts follow a predictable naming scheme so that package manager workflows can construct download URLs without inspecting the build output at runtime.

| Artifact | Naming pattern |
|---|---|
| CLI binary (non-Windows) | `unbill-cli-{platform}` |
| CLI binary (Windows) | `unbill-cli-{platform}.exe` |
| TUI binary (non-Windows) | `unbill-tui-{platform}` |
| TUI binary (Windows) | `unbill-tui-{platform}.exe` |
| Tauri bundle | `unbill-{platform}.{ext}` |

Platform suffixes are `linux-x86_64`, `macos-aarch64`, and `windows-x86_64`.

______________________________________________________________________

## Secrets

| Secret | Used by | Purpose |
|---|---|---|
| `AUR_SSH_KEY` | `release-aur.yml` | SSH private key for the AUR account that owns the AUR packages |

______________________________________________________________________

## Version Management

Versions are managed using `cargo release`. Running `cargo release patch`, `cargo release minor`, or `cargo release major` bumps the version across `Cargo.toml` and `tauri.conf.json`, commits the change, creates a git tag, and pushes. The version release pipeline triggers automatically on the tag push.

The workspace `Cargo.toml` holds a single version shared by all crates via `version.workspace = true`. The `tauri.conf.json` version is kept in sync via the `release-please-config.json` extra-files configuration, which also serves as documentation of which files contain version strings.

`cargo release` is configured via `release.toml` to use a single workspace-level tag (`v{version}`) rather than per-crate tags, and to skip publishing to crates.io.
