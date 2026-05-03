use std::process::Command;

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub issue_found: bool,
    pub output: String,
}

pub fn run_detection(command: &str) -> DetectionResult {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            let combined = format!("{}{}", stdout, stderr).trim().to_string();

            // Exit code 0 means the issue was detected (problem exists)
            // Non-zero exit code means no issue found (system is fine)
            DetectionResult {
                issue_found: result.status.success(),
                output: combined,
            }
        }
        Err(e) => DetectionResult {
            issue_found: false,
            output: format!("Failed to run detection: {}", e),
        },
    }
}
