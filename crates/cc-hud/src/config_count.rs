use std::path::Path;

/// Count configuration resources (CLAUDE.md files, rules, MCPs, hooks).
pub fn count_configs(cwd: Option<&str>) -> (usize, usize, usize, usize) {
    let cwd = match cwd {
        Some(c) => Path::new(c),
        None => return (0, 0, 0, 0),
    };

    let claude_md = count_claude_md_files(cwd);
    let (rules, mcps, hooks) = count_settings(cwd);

    (claude_md, rules, mcps, hooks)
}

fn count_claude_md_files(project_dir: &Path) -> usize {
    let mut count = 0;

    // Project-level CLAUDE.md
    if project_dir.join("CLAUDE.md").exists() {
        count += 1;
    }

    // .claude/CLAUDE.md
    if project_dir.join(".claude").join("CLAUDE.md").exists() {
        count += 1;
    }

    // User-level ~/.claude/CLAUDE.md
    if let Some(home) = dirs_home() {
        if home.join(".claude").join("CLAUDE.md").exists() {
            count += 1;
        }
    }

    count
}

fn count_settings(project_dir: &Path) -> (usize, usize, usize) {
    let mut rules = 0;
    let mut mcps = 0;
    let mut hooks = 0;

    // Read project .claude/settings.json
    let project_settings = project_dir.join(".claude").join("settings.json");
    if let Ok(content) = std::fs::read_to_string(&project_settings) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            rules += count_rules(&val);
            mcps += count_mcps(&val);
            hooks += count_hooks(&val);
        }
    }

    // Read user-level ~/.claude/settings.json
    if let Some(home) = dirs_home() {
        let user_settings = home.join(".claude").join("settings.json");
        if let Ok(content) = std::fs::read_to_string(&user_settings) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                rules += count_rules(&val);
                mcps += count_mcps(&val);
                hooks += count_hooks(&val);
            }
        }
    }

    (rules, mcps, hooks)
}

fn count_rules(val: &serde_json::Value) -> usize {
    val.get("rules")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

fn count_mcps(val: &serde_json::Value) -> usize {
    val.get("mcpServers")
        .and_then(|v| v.as_object())
        .map(|o| o.len())
        .unwrap_or(0)
}

fn count_hooks(val: &serde_json::Value) -> usize {
    val.get("hooks")
        .and_then(|v| v.as_object())
        .map(|o| o.values().filter_map(|v| v.as_array()).map(|a| a.len()).sum::<usize>())
        .unwrap_or(0)
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(std::path::PathBuf::from)
}
