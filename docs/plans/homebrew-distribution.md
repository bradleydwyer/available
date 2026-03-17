Status: In Progress

# Homebrew Distribution

## Setup

- Release workflow: `.github/workflows/release.yml`
- Tap repo: `bradleydwyer/homebrew-available`
- Formula: `Formula/available.rb`
- Install: `brew install bradleydwyer/available/available`

## How It Works

On tag push (`v*`):
1. Creates GitHub Release
2. Builds bottles on macOS (arm64: Tahoe, Sequoia, Sonoma)
3. Uploads bottles to the release
4. Updates the homebrew tap formula with bottle SHAs

## Prerequisites

- `TAP_GITHUB_TOKEN` secret set on the `available` repo (PAT with write access to `homebrew-available`)

## Dependencies

Switched from local path deps to git deps so `cargo install` works from a tarball:
- `caucus-core` -> `git = "https://github.com/bradleydwyer/caucus"`
- `parked` -> `git = "https://github.com/bradleydwyer/parked"`
- `staked` -> `git = "https://github.com/bradleydwyer/staked"`

## First Release

Once `TAP_GITHUB_TOKEN` is set:
```bash
git tag v0.1.0
git push origin v0.1.0
```
