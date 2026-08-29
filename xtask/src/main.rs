//! Development tasks for xsteer.
//!
//! Three environments, each with one meaning: dev previews the branch you are
//! working on, beta is always `main`, and a tag is production. Pushing to `main`
//! deploys beta, so the squashed commit is validated there before `release`
//! will tag it — whatever the branch it came from looked like.

use std::env;

mod dev;
mod release;
mod util;
mod version;

const USAGE: &str = "\
usage: cargo xtask <command>

  dev                               deploy the current branch to dev.xsteer.in
  prepare-release <major|minor|patch>   bump the version and open a changelog section
  release [--wait] [--skip-beta-check]  tag main and deploy to xsteer.in";

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("dev") => dev::run_dev(&args[2..]),
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
