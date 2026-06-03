use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub fn run_with_timeout(
    cmd: &str,
    args: &[&str],
    stdin_input: Option<&str>,
    timeout: Duration,
) -> Result<String, String> {
    let mut child = spawn_child(cmd, args)?;
    write_stdin(&mut child, stdin_input)?;
    wait_for_child(&mut child, timeout)
}

fn spawn_child(cmd: &str, args: &[&str]) -> Result<Child, String> {
    Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))
}

fn write_stdin(child: &mut Child, stdin_input: Option<&str>) -> Result<(), String> {
    if let Some(input) = stdin_input {
        let mut stdin = child.stdin.take().ok_or("No stdin available".to_string())?;

        stdin
            .write_all(input.as_bytes())
            .map_err(|e| format!("Stdin write failed: {}", e))?;

        drop(stdin);
    }

    Ok(())
}

fn wait_for_child(child: &mut Child, timeout: Duration) -> Result<String, String> {
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err(format!("Timeout after {:?}", timeout));
        }

        if let Some(exit_status) = child
            .try_wait()
            .map_err(|e| format!("Process error: {}", e))?
        {
            let output = read_stdout(child)?;

            if exit_status.success() {
                return Ok(output);
            } else {
                let error = read_stderr(child)?;
                return Err(format!("Process failed: {}", error));
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn read_stdout(child: &mut Child) -> Result<String, String> {
    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .map_err(|e| format!("Output read failed: {}", e))?;
    Ok(output)
}

fn read_stderr(child: &mut Child) -> Result<String, String> {
    let mut error = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut error)
        .map_err(|e| format!("Error read failed: {}", e))?;
    Ok(error)
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
