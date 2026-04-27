use cc_core::ResourceEntry;
use cc_schema::{Origin, ResourceType};
use ratatui::widgets::TableState;
use std::collections::{HashMap, HashSet};

/// Active tabs in the TUI.
const TAB_NAMES: &[&str] = &["Skills", "Commands", "Agents", "Rules", "Hooks", "Plugins"];
const TAB_TYPES: &[ResourceType] = &[
    ResourceType::Skill,
    ResourceType::Command,
    ResourceType::Agent,
    ResourceType::Rule,
    ResourceType::Hook,
    ResourceType::Plugin,
];

/// Application state for the TUI.
pub struct App {
    /// Currently displayed resources.
    pub resources: Vec<ResourceEntry>,
    /// Active resource type tab.
    pub resource_type: ResourceType,
    /// Whether the detail panel is shown.
    pub show_detail: bool,
    /// Cached snapshot text for the selected resource.
    pub snapshot: Option<String>,
    /// Status bar message.
    pub status_message: String,
    /// Tab index.
    tab_index: usize,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Whether we're showing the quit/install confirmation dialog.
    pub confirm_quit: bool,
    /// Table state for selection tracking.
    pub table_state: TableState,
    /// Indices of rows the user has marked for installation, per resource type.
    selected_indices: HashMap<ResourceType, HashSet<usize>>,
    /// Names of resources physically present in the project's .claude/<type>/ directory.
    project_installed: HashSet<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            resource_type: ResourceType::Skill,
            show_detail: false,
            snapshot: None,
            status_message: String::new(),
            tab_index: 0,
            should_quit: false,
            confirm_quit: false,
            table_state: TableState::default(),
            selected_indices: HashMap::new(),
            project_installed: HashSet::new(),
        }
    }

    /// Load resources for the current tab from the given project/workspace.
    pub fn refresh(&mut self, project_dir: &std::path::Path, workspace_root: &std::path::Path) {
        self.selected_indices.remove(&self.resource_type);

        if self.resource_type == ResourceType::Hook {
            self.resources = Vec::new();
            self.project_installed.clear();
            self.load_hooks_as_entries(project_dir);
            self.table_state.select(Some(0));
            self.show_detail = false;
            self.snapshot = None;
            let count = self.resources.len();
            self.status_message = format!("Loaded {count} hook(s) [read-only]");
            return;
        }

        if self.resource_type == ResourceType::Plugin {
            self.resources = Vec::new();
            self.project_installed.clear();
            self.load_plugins(workspace_root);
            self.table_state.select(Some(0));
            self.show_detail = false;
            self.snapshot = None;
            let count = self.resources.len();
            self.status_message = format!("Loaded {count} plugin(s) [read-only]");
            return;
        }

        // Scan project .claude/<type>/ to find what's physically installed
        self.project_installed = scan_project_resources(project_dir, self.resource_type);

        let extern_libs = cc_core::list_extern_libs(project_dir);
        let mut entries =
            cc_core::discover_resources(self.resource_type, workspace_root, &extern_libs);
        cc_core::resolve_resources(&mut entries);
        self.resources = entries;
        self.table_state.select(Some(0));
        self.show_detail = false;
        self.snapshot = None;
        let count = self.resources.len();
        self.status_message = format!("Loaded {count} {}(s)", self.resource_type);
    }

    /// Whether a resource name is physically installed in the project's .claude/<type>/.
    pub fn is_in_project(&self, name: &str) -> bool {
        self.project_installed.contains(name)
    }

    /// Load hooks from settings.json as pseudo ResourceEntry objects.
    fn load_hooks_as_entries(&mut self, project_dir: &std::path::Path) {
        if let Ok(hooks) = cc_core::hook::load_hooks(project_dir) {
            for (event, matchers) in &hooks {
                for matcher in matchers {
                    let pattern = matcher.matcher.as_deref().unwrap_or("*");
                    for hook in &matcher.hooks {
                        self.resources.push(ResourceEntry {
                            name: format!("{event}/{pattern}"),
                            resource_type: ResourceType::Hook,
                            origin: Origin::Project,
                            path: project_dir.join(".claude").join("settings.json"),
                            active: true,
                            description: Some(format!("{} → {}", hook.hook_type, hook.command)),
                            registry: Some("project".to_string()),
                        });
                    }
                }
            }
        }
    }

    /// Load plugins from the claude_plugins directories in cc-workspace.toml.
    fn load_plugins(&mut self, workspace_root: &std::path::Path) {
        let Some(config) = cc_core::WorkspaceConfig::load(workspace_root) else {
            return;
        };

        let mut seen = HashSet::new();

        for (dir, source_label) in config.claude_plugin_dirs() {
            scan_plugin_dir(&dir, &source_label, &mut seen, &mut self.resources);
        }
    }

    /// Move selection down.
    pub fn next(&mut self) {
        if !self.resources.is_empty() {
            let i = match self.table_state.selected() {
                Some(i) => {
                    if i >= self.resources.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.table_state.select(Some(i));
            self.show_detail = false;
            self.snapshot = None;
        }
    }

    /// Move selection up.
    pub fn previous(&mut self) {
        if !self.resources.is_empty() {
            let i = match self.table_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.resources.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.table_state.select(Some(i));
            self.show_detail = false;
            self.snapshot = None;
        }
    }

    /// Switch to the next resource type tab.
    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % TAB_TYPES.len();
        self.resource_type = TAB_TYPES[self.tab_index];
    }

    /// Switch to the previous resource type tab.
    pub fn prev_tab(&mut self) {
        if self.tab_index == 0 {
            self.tab_index = TAB_TYPES.len() - 1;
        } else {
            self.tab_index -= 1;
        }
        self.resource_type = TAB_TYPES[self.tab_index];
    }

    /// Toggle the detail panel for the selected resource.
    pub fn toggle_detail(&mut self) {
        if self.resources.is_empty() {
            return;
        }
        self.show_detail = !self.show_detail;
        if self.show_detail {
            self.take_snapshot();
        } else {
            self.snapshot = None;
        }
    }

    /// Toggle selection on the current row.
    pub fn toggle_selected(&mut self) {
        if self.resource_type == ResourceType::Hook {
            self.status_message = "Hooks cannot be installed via TUI".to_string();
            return;
        }
        if let Some(i) = self.table_state.selected() {
            if i < self.resources.len() {
                let entry = &self.resources[i];
                if self.is_in_project(&entry.name) {
                    self.status_message =
                        format!("{} '{}' is already installed", entry.resource_type, entry.name);
                    return;
                }
                let selections = self.current_selections_mut();
                if selections.contains(&i) {
                    selections.remove(&i);
                } else {
                    selections.insert(i);
                }
            }
        }
    }

    /// Select all non-active resources.
    pub fn select_all(&mut self) {
        if self.resource_type == ResourceType::Hook {
            return;
        }
        let new_selections: HashSet<usize> = self
            .resources
            .iter()
            .enumerate()
            .filter(|(_, entry)| !self.is_in_project(&entry.name))
            .map(|(i, _)| i)
            .collect();
        let count = new_selections.len();
        *self.current_selections_mut() = new_selections;
        self.status_message = format!("Selected {count} resource(s)");
    }

    /// Clear all selections on the current tab.
    pub fn clear_selection(&mut self) {
        self.current_selections_mut().clear();
        self.status_message = "Selection cleared".to_string();
    }

    /// Number of currently selected items on the current tab.
    pub fn selected_count(&self) -> usize {
        self.selected_indices
            .get(&self.resource_type)
            .map_or(0, |s| s.len())
    }

    /// Whether a given row index is selected on the current tab.
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected_indices
            .get(&self.resource_type)
            .map_or(false, |s| s.contains(&index))
    }

    /// Total number of selected items across all tabs.
    pub fn total_selected_count(&self) -> usize {
        self.selected_indices.values().map(|s| s.len()).sum()
    }

    /// Get a mutable reference to the selection set for the current tab.
    fn current_selections_mut(&mut self) -> &mut HashSet<usize> {
        self.selected_indices.entry(self.resource_type).or_default()
    }

    /// Install all selected resources across all tabs.
    pub fn install_all_selected(
        &mut self,
        project_dir: &std::path::Path,
        workspace_root: &std::path::Path,
    ) {
        let all_selections: Vec<(ResourceType, Vec<usize>)> = self
            .selected_indices
            .iter()
            .filter(|(rt, indices)| !indices.is_empty() && **rt != ResourceType::Hook)
            .map(|(rt, indices)| (*rt, indices.iter().copied().collect()))
            .collect();

        if all_selections.is_empty() {
            self.status_message =
                "No resources selected across any tab. Press Space to select.".to_string();
            return;
        }

        let mut total_installed = 0usize;
        let mut total_skipped = 0usize;
        let mut errors = Vec::new();

        for (rt, indices) in &all_selections {
            let extern_libs = cc_core::list_extern_libs(project_dir);
            let mut entries = cc_core::discover_resources(*rt, workspace_root, &extern_libs);
            cc_core::resolve_resources(&mut entries);

            for idx in indices {
                if let Some(entry) = entries.get(*idx) {
                    match install_resource_to_project(entry, project_dir, workspace_root) {
                        Ok(_) => total_installed += 1,
                        Err(e) => {
                            total_skipped += 1;
                            errors.push(format!("{}: {e}", entry.name));
                        }
                    }
                }
            }
        }

        self.selected_indices.clear();

        if errors.is_empty() {
            self.status_message =
                format!("Installed {total_installed} resource(s) across all tabs");
        } else {
            self.status_message = format!(
                "Installed {total_installed}, {total_skipped} failed. First error: {}",
                errors[0]
            );
        }
    }

    /// Install all selected resources to the project.
    pub fn install_selected(
        &mut self,
        project_dir: &std::path::Path,
        workspace_root: &std::path::Path,
    ) {
        if self.resource_type == ResourceType::Hook {
            self.status_message = "Hooks cannot be installed via TUI".to_string();
            return;
        }

        let indices: Vec<usize> = self
            .selected_indices
            .get(&self.resource_type)
            .map_or(Vec::new(), |s| s.iter().copied().collect());

        if indices.is_empty() {
            self.status_message = "No resources selected. Press Space to select.".to_string();
            return;
        }

        let mut installed = 0usize;
        let mut skipped = 0usize;
        let mut errors = Vec::new();

        for idx in &indices {
            if let Some(entry) = self.resources.get(*idx) {
                match install_resource_to_project(entry, project_dir, workspace_root) {
                    Ok(_dest) => {
                        installed += 1;
                        // Mark as active in our local state
                        self.resources[*idx].active = true;
                    }
                    Err(e) => {
                        skipped += 1;
                        errors.push(format!("{}: {e}", entry.name));
                    }
                }
            }
        }

        self.selected_indices.remove(&self.resource_type);

        if errors.is_empty() {
            self.status_message = format!("Installed {installed} resource(s) successfully");
        } else {
            self.status_message =
                format!("Installed {installed}, {skipped} failed. First error: {}", errors[0]);
        }
    }

    /// Build a snapshot string for the selected resource.
    fn take_snapshot(&mut self) {
        if let Some(i) = self.table_state.selected() {
            if let Some(entry) = self.resources.get(i) {
                let active = if entry.active { "active" } else { "inactive" };
                let registry = entry.registry.as_deref().unwrap_or("-");
                let desc = entry.description.as_deref().unwrap_or("-");
                let selected = if self.is_selected(i) {
                    "marked for install"
                } else {
                    ""
                };
                self.snapshot = Some(format!(
                    "Name: {}\nType: {}\nOrigin: {}\nRegistry: {}\nActive: {}\nSelected: {}\nDescription: {}\nPath: {}",
                    entry.name,
                    entry.resource_type,
                    entry.origin,
                    registry,
                    active,
                    if selected.is_empty() { "no" } else { selected },
                    desc,
                    entry.path.display(),
                ));
            }
        }
    }

    /// Request quit — shows confirmation if there are selected resources.
    pub fn quit(&mut self) {
        if self.total_selected_count() > 0 && !self.confirm_quit {
            self.confirm_quit = true;
        } else {
            self.should_quit = true;
        }
    }

    /// Cancel the quit confirmation dialog.
    pub fn cancel_quit(&mut self) {
        self.confirm_quit = false;
    }

    /// Confirm quit and install all selected resources.
    pub fn confirm_and_install(
        &mut self,
        project_dir: &std::path::Path,
        workspace_root: &std::path::Path,
    ) {
        self.install_all_selected(project_dir, workspace_root);
        self.should_quit = true;
    }

    /// Confirm quit without installing (discard selections).
    pub fn confirm_without_install(&mut self) {
        self.selected_indices.clear();
        self.should_quit = true;
    }

    /// Get the tab names for rendering.
    pub fn tab_names() -> &'static [&'static str] {
        TAB_NAMES
    }

    /// Current tab index.
    pub fn current_tab_index(&self) -> usize {
        self.tab_index
    }
}

/// Install a single resource to the project by copying its source directory/file.
fn install_resource_to_project(
    entry: &ResourceEntry,
    project_dir: &std::path::Path,
    _workspace_root: &std::path::Path,
) -> Result<(), String> {
    let resource_type = entry.resource_type;
    let target_dir = cc_core::paths::resource_dir(project_dir, resource_type);
    let dest = target_dir.join(&entry.name);

    // Source: the parent directory for subdirectory-style resources, or the file itself
    let source_path = &entry.path;

    if resource_type == ResourceType::Rule {
        // Rules are single .md files
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("create dir: {e}"))?;
        let dest_file = target_dir.join(format!("{}.md", entry.name));
        std::fs::copy(source_path, &dest_file)
            .map_err(|e| format!("copy: {e}"))?;
        return Ok(());
    }

    // For skills/commands/agents — handle both flat .md files and subdirectory patterns
    let source_dir = source_path
        .parent()
        .unwrap_or(std::path::Path::new("."));

    // If the source is a marker file (e.g., SKILL.md inside a named dir), copy the whole dir
    let is_marker = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| matches!(name, "SKILL.md" | "COMMAND.md" | "AGENT.md"));

    if is_marker {
        // Copy entire parent directory
        if dest.exists() {
            std::fs::remove_dir_all(&dest).map_err(|e| format!("remove old: {e}"))?;
        }
        std::fs::create_dir_all(&dest)
            .map_err(|e| format!("create dir: {e}"))?;
        copy_dir_recursive(source_dir, &dest)?;
    } else {
        // Copy single .md file
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("create dir: {e}"))?;
        let dest_file = target_dir.join(source_path.file_name().unwrap_or_default());
        std::fs::copy(source_path, &dest_file)
            .map_err(|e| format!("copy: {e}"))?;
    }

    Ok(())
}

/// Recursively copy a directory's contents into the destination.
fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    for entry in std::fs::read_dir(src).map_err(|e| format!("read dir: {e}"))? {
        let entry = entry.map_err(|e| format!("read entry: {e}"))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            std::fs::create_dir_all(&dest_path).map_err(|e| format!("create: {e}"))?;
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path).map_err(|e| format!("copy: {e}"))?;
        }
    }
    Ok(())
}

/// Scan the project's .claude/<type>/ directory and return the set of installed resource names.
fn scan_project_resources(
    project_dir: &std::path::Path,
    resource_type: ResourceType,
) -> HashSet<String> {
    let dir = cc_core::paths::resource_dir(project_dir, resource_type);
    let mut names = HashSet::new();

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return names;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Subdirectory pattern: skills/foo/SKILL.md → name is directory name
            let marker = match resource_type {
                ResourceType::Skill => "SKILL.md",
                ResourceType::Command => "COMMAND.md",
                ResourceType::Agent => "AGENT.md",
                _ => continue,
            };
            if path.join(marker).exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    names.insert(name.to_string());
                }
            }
        } else if path.extension().is_some_and(|ext| ext == "md") {
            // Flat file: skills/foo.md → name is file stem
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.insert(stem.to_string());
            }
        }
    }

    names
}

/// Scan a plugin directory for subdirectories, each representing a plugin.
fn scan_plugin_dir(
    dir: &std::path::Path,
    source_label: &str,
    seen: &mut HashSet<String>,
    resources: &mut Vec<ResourceEntry>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match entry.file_name().to_str() {
            Some(n) if !n.starts_with('.') => n.to_string(),
            _ => continue,
        };
        if !seen.insert(name.clone()) {
            continue;
        }

        let description = read_plugin_description(&path);

        resources.push(ResourceEntry {
            name,
            resource_type: ResourceType::Plugin,
            origin: Origin::User,
            path,
            active: true,
            description,
            registry: Some(source_label.to_string()),
        });
    }
}

/// Read the first paragraph from README.md in a plugin directory as its description.
fn read_plugin_description(plugin_dir: &std::path::Path) -> Option<String> {
    let readme_path = plugin_dir.join("README.md");
    let content = std::fs::read_to_string(&readme_path).ok()?;

    // Find the first non-empty, non-heading line as the description.
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}
