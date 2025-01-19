use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub fn run_with_timeout(
    cmd: &str,
    args: &[&str],
    stdin_input: Option<&str>,
    timeout: Duration,
) -> Result<String, String> {
    // Spawn child process
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    // Write to stdin if provided
    if let Some(input) = stdin_input {
        let mut stdin = child.stdin.take().ok_or("No stdin available".to_string())?;

        stdin
            .write_all(input.as_bytes())
            .map_err(|e| format!("Stdin write failed: {}", e))?;

        drop(stdin);
    }

    // Shared flag for completion
    let child = Arc::new(Mutex::new(child));
    let start = Instant::now();

    // Try waiting in a loop
    loop {
        // Check timeout
        if start.elapsed() > timeout {
            let _ = child.lock().unwrap().kill();
            return Err(format!("Timeout after {:?}", timeout));
        }

        // Check if process completed
        let status = child
            .lock()
            .unwrap()
            .try_wait()
            .map_err(|e| format!("Process error: {}", e))?;

        if let Some(exit_status) = status {
            // Process finished, read output
            let mut output = String::new();
            child
                .lock()
                .unwrap()
                .stdout
                .take()
                .unwrap()
                .read_to_string(&mut output)
                .map_err(|e| format!("Output read failed: {}", e))?;

            if exit_status.success() {
                return Ok(output);
            } else {
                let mut error = String::new();
                child
                    .lock()
                    .unwrap()
                    .stderr
                    .take()
                    .unwrap()
                    .read_to_string(&mut error)
                    .map_err(|e| format!("Error read failed: {}", e))?;
                return Err(format!("Process failed: {}", error));
            }
        }

        // Sleep to prevent busy waiting
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn test_run_with_timeout() {
    let result_1 = run_with_timeout("echo", &["1"], None, Duration::from_millis(100));
    assert!(result_1.is_ok());
    let output_1 = result_1.unwrap();
    assert_eq!(output_1, "1\n");

    let result_2 = run_with_timeout("sort", &[], Some("b\na"), Duration::from_millis(100));
    assert!(result_2.is_ok());
    let output_2 = result_2.unwrap();
    assert_eq!(output_2, "a\nb\n");

    let result_3 = run_with_timeout("sleep", &["5"], None, Duration::from_millis(10));
    assert!(result_3.is_err());
    assert_eq!(result_3.unwrap_err(), "Timeout after 10ms");
}
