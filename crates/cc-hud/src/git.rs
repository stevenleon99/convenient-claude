use std::process::Command;

use crate::types::GitStatus;

/// Get git status for a directory. Shells out to `git`.
pub fn get_git_status(cwd: Option<&str>) -> Option<GitStatus> {
    let cwd = cwd?;

    let branch = get_branch(cwd)?;
    let is_dirty = get_dirty(cwd);
    let (ahead, behind) = get_ahead_behind(cwd);

    Some(GitStatus {
        branch,
        is_dirty,
        ahead,
        behind,
    })
}

fn get_branch(cwd: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        return None;
    }

    Some(branch)
}

fn get_dirty(cwd: &str) -> bool {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(cwd)
        .output();

    match output {
        Ok(out) if out.status.success() => !out.stdout.is_empty(),
        _ => false,
    }
}

fn get_ahead_behind(cwd: &str) -> (u32, u32) {
    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "@{upstream}...HEAD"])
        .current_dir(cwd)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = s.split_whitespace().collect();
            let behind = parts.first().and_then(|v| v.parse().ok()).unwrap_or(0);
            let ahead = parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
            (ahead, behind)
        }
        _ => (0, 0),
    }
}
