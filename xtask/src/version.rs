//! The workspace version is the single source of truth for what a release is
//! called. It is read here and never typed at a prompt, which is what keeps
//! `Cargo.toml` and the git tag from drifting apart.

use crate::util::fail;
use std::fs;

pub const CARGO_TOML: &str = "Cargo.toml";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Bump {
    Major,
    Minor,
    Patch,
}

impl Bump {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "major" => Some(Self::Major),
            "minor" => Some(Self::Minor),
            "patch" => Some(Self::Patch),
            _ => None,
        }
    }
}

/// Reads `[workspace.package] version`. Scoped to that table on purpose: the
/// file is full of `version = "..."` lines under `[workspace.dependencies]`,
/// and a looser match would happily return serde's version.
pub fn current() -> String {
    let content = fs::read_to_string(CARGO_TOML)
        .unwrap_or_else(|e| fail(format!("could not read {CARGO_TOML}: {e}")));

    parse(&content).unwrap_or_else(|| fail("no `version` under [workspace.package] in Cargo.toml"))
}

/// Rewrites `[workspace.package] version` to `new_version`, leaving every other
/// byte of the file alone.
pub fn write(new_version: &str) {
    let content = fs::read_to_string(CARGO_TOML)
        .unwrap_or_else(|e| fail(format!("could not read {CARGO_TOML}: {e}")));

    let out = replace(&content, new_version)
        .unwrap_or_else(|| fail("no `version` under [workspace.package] in Cargo.toml"));

    fs::write(CARGO_TOML, out)
        .unwrap_or_else(|e| fail(format!("could not write {CARGO_TOML}: {e}")));
}

/// Rewrites only the first `version` line inside `[workspace.package]`.
fn replace(content: &str, new_version: &str) -> Option<String> {
    let mut out = String::with_capacity(content.len());
    let mut in_workspace_package = false;
    let mut replaced = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_workspace_package = trimmed == "[workspace.package]";
        }

        if in_workspace_package && !replaced && trimmed.starts_with("version = \"") {
            out.push_str(&format!("version = \"{new_version}\"\n"));
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }

    replaced.then_some(out)
}

pub fn bumped(current: &str, bump: Bump) -> String {
    let parts: Vec<&str> = current.split('.').collect();
    if parts.len() != 3 {
        fail(format!("version `{current}` is not major.minor.patch"));
    }

    let num = |i: usize| -> u64 {
        parts[i]
            .parse()
            .unwrap_or_else(|_| fail(format!("version `{current}` is not numeric")))
    };
    let (major, minor, patch) = (num(0), num(1), num(2));

    match bump {
        Bump::Major => format!("{}.0.0", major + 1),
        Bump::Minor => format!("{major}.{}.0", minor + 1),
        Bump::Patch => format!("{major}.{minor}.{}", patch + 1),
    }
}

/// Reads the first `version` inside `[workspace.package]`.
fn parse(content: &str) -> Option<String> {
    let mut in_workspace_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_workspace_package = trimmed == "[workspace.package]";
        }
        if in_workspace_package && trimmed.starts_with("version = \"") {
            let start = trimmed.find('"')? + 1;
            let end = trimmed.rfind('"')?;
            return Some(trimmed[start..end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shaped like the real file: a `[workspace.dependencies]` table below whose
    /// entries also carry `version = "..."`. Matching loosely returns serde's.
    const SAMPLE: &str = r#"[workspace]
resolver = "2"
members = ["core", "wasm", "xtask"]

[workspace.package]
version = "0.2.0"
edition = "2021"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
regex = "1.10"
"#;

    #[test]
    fn reads_the_workspace_package_version() {
        assert_eq!(parse(SAMPLE).as_deref(), Some("0.2.0"));
    }

    #[test]
    fn ignores_dependency_versions() {
        let no_package_version = "[workspace.dependencies]\nserde = { version = \"1.0\" }\n";
        assert_eq!(parse(no_package_version), None);
    }

    #[test]
    fn replaces_only_the_workspace_package_version() {
        let out = replace(SAMPLE, "0.3.0").expect("should replace");
        assert!(out.contains("version = \"0.3.0\""));
        // The dependency table is left exactly as it was.
        assert!(out.contains("serde = { version = \"1.0\", features = [\"derive\"] }"));
        assert_eq!(parse(&out).as_deref(), Some("0.3.0"));
    }

    #[test]
    fn replace_reports_a_missing_version() {
        assert_eq!(replace("[workspace]\nresolver = \"2\"\n", "1.0.0"), None);
    }

    #[test]
    fn bumps_each_component() {
        assert_eq!(bumped("0.2.3", Bump::Major), "1.0.0");
        assert_eq!(bumped("0.2.3", Bump::Minor), "0.3.0");
        assert_eq!(bumped("0.2.3", Bump::Patch), "0.2.4");
    }

    #[test]
    fn bumping_rolls_over() {
        assert_eq!(bumped("1.9.9", Bump::Minor), "1.10.0");
        assert_eq!(bumped("9.9.9", Bump::Major), "10.0.0");
    }
}
