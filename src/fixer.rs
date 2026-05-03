use std::process::Command;

#[derive(Debug, Clone)]
pub struct FixResult {
    pub success: bool,
    pub output: String,
}

pub fn run_fix(command: &str) -> FixResult {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            let combined = format!("{}{}", stdout, stderr).trim().to_string();

            FixResult {
                success: result.status.success(),
                output: combined,
            }
        }
        Err(e) => FixResult {
            success: false,
            output: format!("Failed to run fix: {}", e),
        },
    }
}
