# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Releases are cut with `cargo xtask prepare-release <major|minor|patch>`, which
opens a dated section below. See [`docs/DEPLOY.md`](docs/DEPLOY.md).

## [Unreleased]

## [0.2.2] - 2026-08-29

### Added

- The footer carries the build's version and short commit, linked to that
  commit on GitHub, so a bug report can name the build it came from. Both are
  baked in at build time and degrade independently: CI supplies `GITHUB_SHA`,
  a local build asks git and marks a dirty tree with `*`, and a checkout
  without git history simply omits the commit.
- The version shown comes from `[workspace.package]` in `Cargo.toml` — the
  same line `cargo xtask release` tags from — rather than adding a third place
  a version has to be remembered.

### Fixed

- The host badge only recognised beta, so `dev.xsteer.in` rendered no badge at
  all and was indistinguishable from production. It now reports dev and beta
  alike.
- `cargo xtask release` treated an in-flight beta deploy as a failure: `gh`
  reports an unfinished run's conclusion as an empty string, not null, so the
  "still running" branch never fired. Classification now keys on `status`,
  which means `--wait` waits instead of giving up in exactly the case it
  exists for.

### Removed

- The footer's repeated list of repositories. The Open source section already
  introduces all three with the context that makes them worth clicking.

## [0.2.1] - 2026-08-29

### Added

- A third environment, `dev.xsteer.in`, for previewing whatever branch is being
  worked on. Beta now serves `main` and only `main`, so the two roles it used to
  share — unstable preview and release candidate — are separated. The Deploy Beta
  workflow refuses a dispatch against any other ref rather than letting a branch
  quietly take beta's place.
- `xtask` with three commands: `dev` deploys the current branch to
  dev.xsteer.in, `prepare-release` bumps the version and opens a changelog
  section, and `release` tags `main`. The tag is derived from
  `[workspace.package] version`, never typed, so the two cannot drift.
- `release` refuses to tag a commit unless a Deploy Beta run for that exact SHA
  concluded successfully, which is what makes beta a gate rather than a habit.
  Squash merges are covered by the automatic `main` → beta deploy; `--wait`
  blocks for it rather than making you poll.

### Changed

- Tests now run on every branch push, not only on pull requests and `main`.
  With previews happening pre-merge, waiting for a PR meant nothing ran until
  after the code had landed.

## [0.2.0] - 2026-08-28

### Added

- Dark/light theme toggle in the navbar, persisted to `localStorage`. An unset
  preference follows the operating system, including live changes.
- An Open source section covering Xfina, Xfingine and Xsteer.
- A "Privacy first" header introducing the privacy section.

### Changed

- The theme is applied before first paint from `index.html`, replacing the
  deferred module bootstrap that let a white flash through on dark-mode
  machines.
- Anchored sections carry `scroll-mt-3`; the sticky header was covering each
  heading by 5px after an anchor jump.
- Axis added to the credit cards read; brokerage relabelled "International
  brokerage".
- The footer lists the three repositories as flat links.

### Removed

- The redundant "parsing is powered by Xfina" line, now that the Open source
  section credits it.

## [0.1.0] - 2026-08-22

### Added

- Initial scaffold: the `core` domain model, the static Vite + Tailwind site,
  and an assets-only Cloudflare Worker deployed from GitHub Actions.
- Scripted Cloudflare and Spaceship DNS setup (`scripts/setup-dns.sh`).
- Beta and production deploy workflows, with `DEPLOY_ENV` defaulting to a
  noindex build so a misconfigured workflow costs a deploy rather than the
  domain's search presence.

[Unreleased]: https://github.com/sakthipriyan/xsteer/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/sakthipriyan/xsteer/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/sakthipriyan/xsteer/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/sakthipriyan/xsteer/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/sakthipriyan/xsteer/releases/tag/v0.1.0
