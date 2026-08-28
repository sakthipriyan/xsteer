//! Shared plumbing: shelling out, and the git questions every command asks.

use std::process::{exit, Command};

pub const MAIN: &str = "main";

/// Every failure in a release path is a stop, never a warning. Half-finished
/// releases are worse than no release.
pub fn fail(msg: impl AsRef<str>) -> ! {
    eprintln!("error: {}", msg.as_ref());
    exit(1)
}

/// Runs a command, echoing it first so the terminal doubles as an audit trail of
/// what was done to the repository.
pub fn run(cmd: &str, args: &[&str]) {
    println!("> {} {}", cmd, args.join(" "));
    let status = Command::new(cmd)
        .args(args)
        .status()
        .unwrap_or_else(|e| fail(format!("could not run `{cmd}`: {e}")));

    if !status.success() {
        fail(format!("`{cmd} {}` failed", args.join(" ")));
    }
}

/// Runs a command and returns trimmed stdout. Stderr is left attached so the
/// underlying tool can explain itself.
pub fn capture(cmd: &str, args: &[&str]) -> String {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .unwrap_or_else(|e| fail(format!("could not run `{cmd}`: {e}")));

    if !out.status.success() {
        fail(format!("`{cmd} {}` failed", args.join(" ")));
    }
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// True when the command exits zero. For questions, not actions.
pub fn succeeds(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn current_branch() -> String {
    capture("git", &["rev-parse", "--abbrev-ref", "HEAD"])
}

pub fn head_sha() -> String {
    capture("git", &["rev-parse", "HEAD"])
}

/// A dirty tree means the thing you are about to publish is not the thing in
/// front of you.
pub fn ensure_clean() {
    if !capture("git", &["status", "--porcelain"]).is_empty() {
        fail("working tree is not clean — commit or set aside your changes first");
    }
}

pub fn ensure_on_main() {
    let branch = current_branch();
    if branch != MAIN {
        fail(format!(
            "expected to be on `{MAIN}`, but this is `{branch}`"
        ));
    }
}

/// Fetches, then refuses to continue unless the local branch is exactly the
/// remote one. Tagging a stale local `main` produces a release nobody can find.
pub fn ensure_synced_with_origin(branch: &str) {
    run("git", &["fetch", "origin", branch, "--tags"]);

    let local = capture("git", &["rev-parse", branch]);
    let remote = capture("git", &["rev-parse", &format!("origin/{branch}")]);

    if local != remote {
        fail(format!(
            "`{branch}` and `origin/{branch}` differ ({} vs {}) — push or pull before continuing",
            &local[..7],
            &remote[..7]
        ));
    }
}

pub fn ensure_gh_available() {
    if !succeeds("gh", &["auth", "status"]) {
        fail("the GitHub CLI is required — install it and run `gh auth login`");
    }
}
