use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

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

pub struct ProgressFixHandle {
    pub result_receiver: Receiver<FixResult>,
    pub progress_receiver: Receiver<i32>,
}

pub fn run_fix_with_progress(command: String) -> ProgressFixHandle {
    let (result_tx, result_rx) = mpsc::channel();
    let (progress_tx, progress_rx) = mpsc::channel();

    thread::spawn(move || {
        // Spawn another thread to send progress updates while command runs
        let progress_tx_clone = progress_tx.clone();
        let command_clone = command.clone();

        let progress_thread = thread::spawn(move || {
            // Simulate progress with increasing delays
            // Start fast, slow down near the end
            let mut progress = 0i32;

            while progress < 95 {
                // Calculate delay - faster at start, slower near end
                let delay_ms = match progress {
                    0..=30 => 50,    // Fast start
                    31..=60 => 100,  // Medium
                    61..=80 => 150,  // Slower
                    _ => 200,        // Slow near end
                };

                thread::sleep(Duration::from_millis(delay_ms));

                // Increment with some randomness
                let increment = match progress {
                    0..=30 => 3 + (progress % 3),
                    31..=60 => 2 + (progress % 2),
                    61..=80 => 1,
                    _ => 1,
                };

                progress = (progress + increment).min(95);

                // Send progress update
                let _ = progress_tx_clone.send(progress);
            }
        });

        // Run the actual fix command
        let result = run_fix(&command_clone);

        // Wait for progress thread to get close to 95%
        let _ = progress_thread.join();

        // Send final 100% progress
        let _ = progress_tx.send(100);

        // Small delay to show 100% before showing result
        thread::sleep(Duration::from_millis(300));

        // Send the final result
        let _ = result_tx.send(result);
    });

    ProgressFixHandle {
        result_receiver: result_rx,
        progress_receiver: progress_rx,
    }
}
