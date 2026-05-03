use crate::issue::{Issue, IssueFile};
use std::fs;
use std::path::Path;

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

    fn load_issues(&mut self) {
        let fixes_dir = Path::new("./fixes");

        if !fixes_dir.exists() {
            eprintln!("Warning: fixes/ directory not found");
            return;
        }

        let mut entries: Vec<_> = fs::read_dir(fixes_dir)
            .expect("Failed to read fixes directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "toml")
                    .unwrap_or(false)
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let content = fs::read_to_string(&path).expect("Failed to read TOML file");

            let issue_file: IssueFile =
                toml::from_str(&content).expect(&format!("Failed to parse {:?}", path));

            self.issues.extend(issue_file.issue);
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
