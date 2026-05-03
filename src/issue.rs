use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Issue {
    pub id: String,
    pub category: String,
    pub name: String,
    pub symptoms: Vec<String>,
    pub detection: String,
    pub fix: String,
    pub safe: bool,
}

#[derive(Debug, Deserialize)]
pub struct IssueFile {
    pub issue: Vec<Issue>,
}

impl Issue {
    pub fn icon_for_category(category: &str) -> &'static str {
        match category {
            "gaming" => "🎮",
            "video-conferencing" => "📹",
            "graphics" => "🖥️",
            "audio" => "🔊",
            "display" => "🖥️",
            _ => "🔧",
        }
    }

    pub fn display_name(&self) -> String {
        format!("{} {}", Self::icon_for_category(&self.category), self.name)
    }
}
