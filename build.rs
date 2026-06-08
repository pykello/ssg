use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    if let Ok(head) = std::fs::read_to_string(".git/HEAD") {
        if let Some(reference) = head.strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed=.git/{}", reference.trim());
        }
    }

    println!(
        "cargo:rustc-env=SSG_GIT_SHA={}",
        command_output("git", &["rev-parse", "--short=12", "HEAD"])
    );
    println!(
        "cargo:rustc-env=SSG_BUILD_DATE={}",
        command_output("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"])
    );
}

fn command_output(command: &str, args: &[&str]) -> String {
    Command::new(command)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .filter(|output| !output.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
