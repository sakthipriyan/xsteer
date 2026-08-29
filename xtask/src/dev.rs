//! `cargo xtask dev` — put the current branch on dev.xsteer.in.
//!
//! Dev is the unstable host: it serves whatever branch you last pointed it at,
//! and is expected to churn. Keeping branch previews here is what lets beta mean
//! `main` and nothing else, so a green beta run is real evidence about the
//! commit `release` is about to tag.

use crate::util::{
    capture, current_branch, ensure_clean, ensure_gh_available, fail, run, succeeds,
};
use serde::Deserialize;

const WORKFLOW: &str = "deploy-dev.yml";

#[derive(Deserialize)]
struct Run {
    #[serde(rename = "databaseId")]
    database_id: u64,
    url: String,
    #[serde(rename = "headSha")]
    head_sha: String,
}

pub fn run_dev(args: &[String]) {
    if !args.is_empty() {
        fail("usage: cargo xtask dev");
    }

    ensure_gh_available();
    // Deploying a branch that differs from what is in front of you defeats the
    // purpose of a preview.
    ensure_clean();

    let branch = current_branch();
    let sha = capture("git", &["rev-parse", "HEAD"]);

    println!("Pushing {branch} to origin...");
    run("git", &["push", "--set-upstream", "origin", &branch]);

    println!("Dispatching {WORKFLOW} against {branch}...");
    // GitHub only registers a workflow_dispatch trigger once the file is on the
    // default branch, so a brand-new workflow 404s from every branch including
    // its own. That reads as "the workflow is missing" unless it is spelled out.
    if !succeeds("gh", &["workflow", "view", WORKFLOW]) {
        fail(format!(
            "{WORKFLOW} is not dispatchable yet.\n\n  \
             GitHub only registers workflow_dispatch triggers for workflows on the default \
             branch,\n  so this one has to reach main before it can be used from any branch."
        ));
    }
    run("gh", &["workflow", "run", WORKFLOW, "--ref", &branch]);

    match find_run(&branch, &sha) {
        Some(run) => {
            println!("\nDeploying {} to https://dev.xsteer.in", &sha[..7]);
            println!("  {}", run.url);
            println!("  watch: gh run watch {}", run.database_id);
        }
        None => {
            // The dispatch succeeded; only our attempt to name the run did not.
            println!("\nDispatched. Find the run with:");
            println!("  gh run list --workflow={WORKFLOW} --branch {branch}");
        }
    }
}

/// A dispatched run takes a moment to appear in the API, so poll briefly rather
/// than reporting a miss on the first try.
fn find_run(branch: &str, sha: &str) -> Option<Run> {
    for attempt in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(if attempt == 0 {
            2
        } else {
            3
        }));

        let json = capture(
            "gh",
            &[
                "run",
                "list",
                &format!("--workflow={WORKFLOW}"),
                "--branch",
                branch,
                "--limit",
                "5",
                "--json",
                "databaseId,url,headSha",
            ],
        );

        if let Ok(runs) = serde_json::from_str::<Vec<Run>>(&json) {
            if let Some(run) = runs.into_iter().find(|r| r.head_sha == sha) {
                return Some(run);
            }
        }
    }
    None
}
