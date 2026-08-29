//! `prepare-release` and `release`.
//!
//! The split follows the branch-first flow: the version bump and changelog land
//! on the feature branch, so they are part of what dev previews. `release` then
//! only tags what is already on `main`, gated on beta — which serves `main` and
//! nothing else.

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
    println!("  cargo xtask dev                   # preview it");
    println!("  gh pr create && gh pr merge --squash");
    println!("  git checkout {MAIN} && git pull");
    println!("  cargo xtask release --wait        # waits for main's beta deploy, then tags");
}

pub fn run_release(args: &[String]) {
    let mut skip_beta_check = false;
    let mut wait = false;
    for arg in args {
        match arg.as_str() {
            "--skip-beta-check" => skip_beta_check = true,
            "--wait" => wait = true,
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
        ensure_beta_passed(&sha, wait);
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
///
/// Because beta serves only `main`, the run this finds is always the post-merge
/// one. That is what makes it meaningful under a squash merge: the commit on
/// `main` is a new one no branch preview covered, but beta deployed that exact
/// artifact, which is the one production is about to serve.
fn ensure_beta_passed(sha: &str, wait: bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(600);

    loop {
        match beta_status(sha) {
            BetaStatus::Succeeded => {
                println!("Beta is green for {}.", &sha[..7]);
                return;
            }
            BetaStatus::Failed => fail(format!(
                "the beta deploy for {} did not succeed — fix it before promoting",
                &sha[..7]
            )),
            state => {
                if !wait {
                    fail(match state {
                        BetaStatus::Running => format!(
                            "the beta deploy for {} is still running — re-run with --wait",
                            &sha[..7]
                        ),
                        _ => format!(
                            "no beta deploy found for {}.\n\n  \
                             Pushing to main deploys beta, so after a merge just wait for that \
                             run —\n  `cargo xtask release --wait` blocks until it finishes.",
                            &sha[..7]
                        ),
                    })
                }

                if std::time::Instant::now() >= deadline {
                    fail(format!(
                        "gave up waiting for a beta deploy of {}",
                        &sha[..7]
                    ));
                }
                println!("Waiting for the beta deploy of {}...", &sha[..7]);
                std::thread::sleep(std::time::Duration::from_secs(10));
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum BetaStatus {
    Succeeded,
    Failed,
    Running,
    Missing,
}

fn beta_status(sha: &str) -> BetaStatus {
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

    classify(&runs, sha)
}

/// A re-run turns a red commit green, so any success wins. An unfinished run is
/// never a pass — that is the difference between "beta is fine" and "beta has
/// actually served this commit".
fn classify(runs: &[BetaRun], sha: &str) -> BetaStatus {
    let matching: Vec<&BetaRun> = runs.iter().filter(|r| r.head_sha == sha).collect();

    if matching.is_empty() {
        BetaStatus::Missing
    } else if matching
        .iter()
        .any(|r| r.conclusion.as_deref() == Some("success"))
    {
        BetaStatus::Succeeded
    } else if matching.iter().any(|r| r.conclusion.is_none()) {
        BetaStatus::Running
    } else {
        BetaStatus::Failed
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run(sha: &str, conclusion: Option<&str>) -> BetaRun {
        BetaRun {
            head_sha: sha.to_string(),
            conclusion: conclusion.map(String::from),
        }
    }

    #[test]
    fn a_commit_beta_never_saw_is_missing() {
        let runs = vec![run("aaa", Some("success"))];
        assert!(classify(&runs, "bbb") == BetaStatus::Missing);
    }

    #[test]
    fn a_successful_run_passes() {
        let runs = vec![run("aaa", Some("success"))];
        assert!(classify(&runs, "aaa") == BetaStatus::Succeeded);
    }

    #[test]
    fn an_unfinished_run_is_not_a_pass() {
        let runs = vec![run("aaa", None)];
        assert!(classify(&runs, "aaa") == BetaStatus::Running);
    }

    #[test]
    fn a_failed_run_is_a_failure() {
        let runs = vec![run("aaa", Some("failure"))];
        assert!(classify(&runs, "aaa") == BetaStatus::Failed);
    }

    /// Re-running a failed deploy is the normal fix, so the later success has to
    /// count rather than the earlier failure sticking.
    #[test]
    fn a_rerun_success_beats_an_earlier_failure() {
        let runs = vec![run("aaa", Some("failure")), run("aaa", Some("success"))];
        assert!(classify(&runs, "aaa") == BetaStatus::Succeeded);
    }

    /// Cancelled is a conclusion, but not a passing one.
    #[test]
    fn cancelled_is_not_a_pass() {
        let runs = vec![run("aaa", Some("cancelled"))];
        assert!(classify(&runs, "aaa") == BetaStatus::Failed);
    }
}
