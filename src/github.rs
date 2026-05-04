use serde::Deserialize;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use zip::ZipArchive;

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
        .timeout(Duration::from_secs(10))
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

// Update checking structures
#[derive(Deserialize, Debug)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Deserialize, Debug)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug)]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub download_url: Option<String>,
    pub new_version: Option<String>,
    pub current_version: String,
}

pub fn check_for_updates(current_version: &str) -> UpdateCheckResult {
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            return UpdateCheckResult {
                update_available: false,
                download_url: None,
                new_version: None,
                current_version: current_version.to_string(),
            }
        }
    };

    let url = "https://api.github.com/repos/ZeroiJ/omarchy-doctor-fixes/releases/latest";

    let response = client
        .get(url)
        .header("User-Agent", "omarchy-doctor/0.1")
        .send();

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<Release>() {
                    Ok(release) => {
                        let remote_version = release.tag_name.trim_start_matches('v');
                        let local_version = current_version.trim_start_matches('v');

                        // Simple version comparison (assumes semver format)
                        let is_newer = compare_versions(remote_version, local_version);

                        if is_newer {
                            // Look for fixes.zip in assets
                            let fixes_zip = release.assets.iter()
                                .find(|a| a.name == "fixes.zip")
                                .map(|a| a.browser_download_url.clone());

                            UpdateCheckResult {
                                update_available: fixes_zip.is_some(),
                                download_url: fixes_zip,
                                new_version: Some(release.tag_name),
                                current_version: current_version.to_string(),
                            }
                        } else {
                            UpdateCheckResult {
                                update_available: false,
                                download_url: None,
                                new_version: None,
                                current_version: current_version.to_string(),
                            }
                        }
                    }
                    Err(_) => UpdateCheckResult {
                        update_available: false,
                        download_url: None,
                        new_version: None,
                        current_version: current_version.to_string(),
                    },
                }
            } else {
                UpdateCheckResult {
                    update_available: false,
                    download_url: None,
                    new_version: None,
                    current_version: current_version.to_string(),
                }
            }
        }
        Err(_) => UpdateCheckResult {
            update_available: false,
            download_url: None,
            new_version: None,
            current_version: current_version.to_string(),
        },
    }
}

fn compare_versions(remote: &str, local: &str) -> bool {
    let remote_parts: Vec<u32> = remote
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let local_parts: Vec<u32> = local
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for i in 0..remote_parts.len().max(local_parts.len()) {
        let r = remote_parts.get(i).copied().unwrap_or(0);
        let l = local_parts.get(i).copied().unwrap_or(0);

        if r > l {
            return true;
        } else if r < l {
            return false;
        }
    }

    false // Equal versions
}

pub fn download_and_install(url: &str) -> Result<usize, String> {
    // Create client with timeout
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to create HTTP client: {}", e)),
    };

    // Download the zip file
    let response = client
        .get(url)
        .header("User-Agent", "omarchy-doctor/0.1")
        .send()
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let bytes = response.bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Create target directory
    let target_dir = if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".local/share/omadoctor/fixes")
    } else {
        return Err("Could not determine home directory".to_string());
    };

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Extract zip
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open zip: {}", e))?;

    let mut file_count = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;

        let outpath = target_dir.join(file.name());

        // Only extract .toml files, ignore directories and other files
        if file.name().ends_with(".toml") {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir: {}", e))?;
            }

            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file: {}", e))?;

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read from zip: {}", e))?;
            outfile.write_all(&buffer)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            file_count += 1;
        }
    }

    Ok(file_count)
}

pub fn get_current_version() -> String {
    // Try to read from system VERSION file first
    let system_version_path = PathBuf::from("/usr/share/omadoctor/VERSION");
    if let Ok(version) = fs::read_to_string(&system_version_path) {
        return version.trim().to_string();
    }

    // Fallback to checking local fixes directory for a version marker
    if let Some(home_dir) = dirs::home_dir() {
        let local_version_path = home_dir.join(".local/share/omadoctor/VERSION");
        if let Ok(version) = fs::read_to_string(&local_version_path) {
            return version.trim().to_string();
        }
    }

    // Final fallback: use app version
    env!("CARGO_PKG_VERSION").to_string()
}
