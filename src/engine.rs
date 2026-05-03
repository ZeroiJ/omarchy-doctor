use crate::issue::{Issue, IssueFile};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Engine {
    pub issues: Vec<Issue>,
    pub selected_index: usize,
}

impl Engine {
    pub fn new() -> Self {
        let mut engine = Self {
            issues: Vec::new(),
            selected_index: 0,
        };
        engine.load_issues();
        engine
    }

    fn get_fixes_dirs() -> Vec<PathBuf> {
        let mut dirs = vec![];

        // 1. User-local directory: ~/.local/share/omadoctor/fixes/
        if let Some(home_dir) = dirs::home_dir() {
            let local_dir = home_dir.join(".local/share/omadoctor/fixes");
            dirs.push(local_dir);
        }

        // 2. System-wide directory: /usr/share/omadoctor/fixes/
        dirs.push(PathBuf::from("/usr/share/omadoctor/fixes"));

        // 3. Development fallback: ./fixes/
        dirs.push(PathBuf::from("./fixes"));

        dirs
    }

    fn load_issues(&mut self) {
        let fixes_dirs = Self::get_fixes_dirs();
        let mut found_any = false;

        for fixes_dir in fixes_dirs {
            if fixes_dir.exists() {
                if let Ok(entries) = fs::read_dir(&fixes_dir) {
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
                                self.issues.extend(issue_file.issue);
                                found_any = true;
                            }
                        }
                    }
                }
            }
        }

        if !found_any {
            eprintln!("Warning: No TOML files found in any fixes directory");
            eprintln!("Searched in:");
            for dir in Self::get_fixes_dirs() {
                eprintln!("  - {}", dir.display());
            }
        }

        // Sort alphabetically by name for display
        self.issues.sort_by(|a, b| a.name.cmp(&b.name));
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
