//! `prepare-release` and `release`.
//!
//! The split follows the branch-first flow: the version bump and changelog land
//! on the feature branch, so they are part of what beta previews. `release`
//! then only tags what is already on `main`.

use crate::util::{
    capture, current_branch, ensure_clean, ensure_gh_available, ensure_on_main,
    ensure_synced_with_origin, fail, head_sha, run, succeeds, MAIN,
};
use crate::version::{self, Bump};
use serde::Deserialize;
use std::fs;

const CHANGELOG: &str = "CHANGELOG.md";
const UNRELEASED: &str = "## [Unreleased]";
const BETA_WORKFLOW: &str = "deploy-beta.yml";

pub fn run_prepare(args: &[String]) {
    let bump = args
        .first()
        .and_then(|s| Bump::parse(s))
        .unwrap_or_else(|| fail("usage: cargo xtask prepare-release <major|minor|patch>"));

    ensure_clean();

    // The bump belongs to the feature branch so that beta previews the exact
    // commit that will later be tagged. Bumping on main skips that step.
    let branch = current_branch();
    if branch == MAIN {
        fail(format!(
            "run this on a feature branch, not `{MAIN}` — the bump should be previewed on beta \
             before it is merged"
        ));
    }

    let current = version::current();
    let next = version::bumped(&current, bump);
    println!("Bumping version: {current} -> {next}");

    version::write(&next);
    open_changelog_section(&next);

    // Picks up the new version in the lockfile.
    run("cargo", &["check", "--quiet"]);

    run(
        "git",
        &["add", version::CARGO_TOML, "Cargo.lock", CHANGELOG],
    );
    run(
        "git",
        &["commit", "-m", &format!("chore(release): v{next}")],
    );

    println!("\nPrepared v{next} on {branch}.");
    println!("Next:");
    println!("  cargo xtask beta                 # preview it");
    println!("  git rebase {MAIN} && git checkout {MAIN} && git merge --ff-only {branch}");
    println!("  cargo xtask release              # tag it");
}

pub fn run_release(args: &[String]) {
    let mut skip_beta_check = false;
    for arg in args {
        match arg.as_str() {
            "--skip-beta-check" => skip_beta_check = true,
            other => fail(format!("unknown argument `{other}`")),
        }
    }

    ensure_on_main();
    ensure_clean();
    ensure_synced_with_origin(MAIN);

    let version = version::current();
    let tag = format!("v{version}");
    let sha = head_sha();

    if succeeds(
        "git",
        &["rev-parse", "-q", "--verify", &format!("refs/tags/{tag}")],
    ) {
        fail(format!("tag {tag} already exists locally"));
    }
    if !capture("git", &["ls-remote", "--tags", "origin", &tag]).is_empty() {
        fail(format!("tag {tag} already exists on origin"));
    }

    if skip_beta_check {
        println!("Skipping the beta check by request.");
    } else {
        ensure_gh_available();
        ensure_beta_passed(&sha);
    }

    println!("Tagging {} as {tag}...", &sha[..7]);
    run("git", &["tag", &tag]);
    run("git", &["push", "origin", &tag]);

    println!(
        "\nPushed {tag}. Deploy Production is now running for {}.",
        &sha[..7]
    );
    println!("  gh run watch $(gh run list --workflow=deploy-production.yml --limit 1 --json databaseId --jq '.[0].databaseId')");
}

#[derive(Deserialize)]
struct BetaRun {
    #[serde(rename = "headSha")]
    head_sha: String,
    /// Null while a run is still going. A run in flight is not a pass, so an
    /// absent conclusion must never satisfy the gate.
    #[serde(default)]
    conclusion: Option<String>,
}

/// Beta is only a gate if promotion checks it. This asks whether *this exact
/// commit* ever went green there — not whether beta is currently healthy, which
/// would pass for a commit beta has never seen.
fn ensure_beta_passed(sha: &str) {
    let json = capture(
        "gh",
        &[
            "run",
            "list",
            &format!("--workflow={BETA_WORKFLOW}"),
            "--limit",
            "50",
            "--json",
            "headSha,conclusion",
        ],
    );

    let runs: Vec<BetaRun> = serde_json::from_str(&json)
        .unwrap_or_else(|e| fail(format!("could not read `gh run list` output: {e}")));

    let matching: Vec<&BetaRun> = runs.iter().filter(|r| r.head_sha == sha).collect();

    if matching.is_empty() {
        fail(format!(
            "no beta deploy found for {} — run `cargo xtask beta` on the branch first, or pass \
             --skip-beta-check if you know why it is missing",
            &sha[..7]
        ));
    }

    if !matching
        .iter()
        .any(|r| r.conclusion.as_deref() == Some("success"))
    {
        let still_running = matching.iter().any(|r| r.conclusion.is_none());
        fail(format!(
            "beta deploys for {} exist but none succeeded{} — fix beta before promoting",
            &sha[..7],
            if still_running {
                " (one is still running)"
            } else {
                ""
            }
        ));
    }

    println!("Beta is green for {}.", &sha[..7]);
}

/// Turns the standing `## [Unreleased]` heading into a dated section for this
/// version, leaving a fresh `## [Unreleased]` above it.
fn open_changelog_section(version: &str) {
    let content = fs::read_to_string(CHANGELOG)
        .unwrap_or_else(|e| fail(format!("could not read {CHANGELOG}: {e}")));

    if !content.contains(UNRELEASED) {
        fail(format!("`{UNRELEASED}` not found in {CHANGELOG}"));
    }

    let date = capture("date", &["+%Y-%m-%d"]);
    let replacement = format!("{UNRELEASED}\n\n## [{version}] - {date}");

    fs::write(CHANGELOG, content.replacen(UNRELEASED, &replacement, 1))
        .unwrap_or_else(|e| fail(format!("could not write {CHANGELOG}: {e}")));
}
