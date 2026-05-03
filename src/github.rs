use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GitHubSearchResponse {
    pub items: Vec<GitHubIssue>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GitHubIssue {
    pub title: String,
    pub html_url: String,
    pub state: String, // "open" or "closed"
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub success: bool,
    pub issues: Vec<GitHubIssue>,
    pub error: String,
}

pub fn search_issues(query: &str) -> SearchResult {
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return SearchResult {
                success: false,
                issues: vec![],
                error: format!("Failed to create HTTP client: {}", e),
            }
        }
    };

    // Build search URL - search in basecamp/omarchy repo
    let encoded_query = urlencoding::encode(query);
    let url = format!(
        "https://api.github.com/search/issues?q={}+repo:basecamp/omarchy&per_page=3",
        encoded_query
    );

    let response = client
        .get(&url)
        .header("User-Agent", "omarchy-doctor/0.1")
        .send();

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<GitHubSearchResponse>() {
                    Ok(data) => SearchResult {
                        success: true,
                        issues: data.items,
                        error: String::new(),
                    },
                    Err(e) => SearchResult {
                        success: false,
                        issues: vec![],
                        error: format!("Failed to parse GitHub response: {}", e),
                    },
                }
            } else {
                SearchResult {
                    success: false,
                    issues: vec![],
                    error: format!("GitHub API error: {}", resp.status()),
                }
            }
        }
        Err(_e) => SearchResult {
            success: false,
            issues: vec![],
            error: "Could not reach GitHub (offline?)".to_string(),
        },
    }
}
