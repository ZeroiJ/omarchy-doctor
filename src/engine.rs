use crate::issue::{Issue, IssueFile};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Engine {
    pub issues: Vec<Issue>,
    pub selected_index: usize,
    pub loaded_from: Vec<String>,
    pub total_loaded: usize,
}

impl Engine {
    pub fn new() -> Self {
        let mut engine = Self {
            issues: Vec::new(),
            selected_index: 0,
            loaded_from: Vec::new(),
            total_loaded: 0,
        };
        engine.load_all_issues();
        engine
    }

    fn get_fixes_dirs() -> Vec<(PathBuf, &'static str)> {
        let mut dirs = vec![];

        // 1. User-local directory: ~/.local/share/omadoctor/fixes/ (highest priority)
        if let Some(home_dir) = dirs::home_dir() {
            let local_dir = home_dir.join(".local/share/omadoctor/fixes");
            dirs.push((local_dir, "user-local"));
        }

        // 2. System-wide directory: /usr/share/omadoctor/fixes/
        dirs.push((PathBuf::from("/usr/share/omadoctor/fixes"), "system"));

        // 3. Development fallback: ./fixes/
        dirs.push((PathBuf::from("./fixes"), "development"));

        dirs
    }

    fn load_all_issues(&mut self) {
        let fixes_dirs = Self::get_fixes_dirs();
        let mut all_issues: HashMap<String, Issue> = HashMap::new();
        let mut sources_loaded = Vec::new();

        for (fixes_dir, source_name) in fixes_dirs {
            if fixes_dir.exists() {
                let count = self.load_from_directory(&fixes_dir, &mut all_issues);
                if count > 0 {
                    sources_loaded.push(format!("{} ({} fixes)", source_name, count));
                }
            }
        }

        // Convert hashmap to vec, sorting by name
        let mut issues: Vec<Issue> = all_issues.into_values().collect();
        issues.sort_by(|a, b| a.name.cmp(&b.name));

        self.total_loaded = issues.len();
        self.loaded_from = sources_loaded;
        self.issues = issues;

        // Log what we loaded
        if !self.loaded_from.is_empty() {
            eprintln!("✅ Loaded {} fixes from: {}", 
                self.total_loaded,
                self.loaded_from.join(", ")
            );
        } else {
            eprintln!("⚠️ No fixes loaded. Checked paths:");
            for (dir, name) in Self::get_fixes_dirs() {
                eprintln!("  - {} ({})", dir.display(), name);
            }
        }
    }

    fn load_from_directory(&self, fixes_dir: &PathBuf, issues_map: &mut HashMap<String, Issue>) -> usize {
        let mut count = 0;

        if let Ok(entries) = fs::read_dir(fixes_dir) {
            let mut toml_files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .map(|ext| ext == "toml")
                        .unwrap_or(false)
                })
                .collect();

            toml_files.sort_by_key(|e| e.file_name());

            for entry in toml_files {
                let path = entry.path();
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(issue_file) = toml::from_str::<IssueFile>(&content) {
                        for issue in issue_file.issue {
                            // Insert or replace - higher priority sources are processed first
                            // so they win in the HashMap
                            if !issues_map.contains_key(&issue.id) {
                                issues_map.insert(issue.id.clone(), issue);
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        count
    }

    pub fn move_up(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = self.issues.len().saturating_sub(1);
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.issues.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.issues.len();
    }

    pub fn selected_issue(&self) -> Option<&Issue> {
        self.issues.get(self.selected_index)
    }
}
