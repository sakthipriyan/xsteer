//! `cargo xtask beta` — put the current branch on beta.xsteer.in.
//!
//! Beta is the preview host: it serves whatever branch was last pushed to it,
//! not `main`. Previewing before the merge is the whole point — by the time a
//! problem shows up on `main`, the thing you wanted to stop has already landed.

use crate::util::{capture, current_branch, ensure_clean, ensure_gh_available, fail, run};
use serde::Deserialize;

const WORKFLOW: &str = "deploy-beta.yml";

#[derive(Deserialize)]
struct Run {
    #[serde(rename = "databaseId")]
    database_id: u64,
    url: String,
    #[serde(rename = "headSha")]
    head_sha: String,
}

pub fn run_beta(args: &[String]) {
    if !args.is_empty() {
        fail("usage: cargo xtask beta");
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
    run("gh", &["workflow", "run", WORKFLOW, "--ref", &branch]);

    match find_run(&branch, &sha) {
        Some(run) => {
            println!("\nDeploying {} to https://beta.xsteer.in", &sha[..7]);
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
