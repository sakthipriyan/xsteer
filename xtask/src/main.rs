//! Development tasks for xsteer.
//!
//! The flow these commands serve: work on a branch, preview it on beta, squash
//! it into main, tag the result. Pushing to main deploys beta again, so the
//! squashed commit is itself validated there before `release` will tag it.

use std::env;

mod beta;
mod release;
mod util;
mod version;

const USAGE: &str = "\
usage: cargo xtask <command>

  beta                              deploy the current branch to beta.xsteer.in
  prepare-release <major|minor|patch>   bump the version and open a changelog section
  release [--wait] [--skip-beta-check]  tag main and deploy to xsteer.in";

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("beta") => beta::run_beta(&args[2..]),
        Some("prepare-release") => release::run_prepare(&args[2..]),
        Some("release") => release::run_release(&args[2..]),
        Some(other) => {
            eprintln!("unknown command: {other}\n\n{USAGE}");
            std::process::exit(1);
        }
        None => {
            eprintln!("{USAGE}");
            std::process::exit(1);
        }
    }
}
