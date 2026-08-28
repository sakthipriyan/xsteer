# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Releases are cut with `cargo xtask prepare-release <major|minor|patch>`, which
opens a dated section below. See [`docs/DEPLOY.md`](docs/DEPLOY.md).

## [Unreleased]

### Added

- `xtask` with three commands: `beta` deploys the current branch to
  beta.xsteer.in, `prepare-release` bumps the version and opens a changelog
  section, and `release` tags `main`. The tag is derived from
  `[workspace.package] version`, never typed, so the two cannot drift.
- `release` refuses to tag a commit unless a Deploy Beta run for that exact SHA
  concluded successfully, which is what makes beta a gate rather than a habit.

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

[Unreleased]: https://github.com/sakthipriyan/xsteer/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/sakthipriyan/xsteer/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/sakthipriyan/xsteer/releases/tag/v0.1.0
