use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temporary_directory() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be valid")
        .as_nanos();
    std::env::temp_dir().join(format!("cwctl-cli-test-{suffix}"))
}

fn run(arguments: &[&str], data_dir: &PathBuf) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cwctl"))
        .args(arguments)
        .env("CWCTL_HOME", data_dir)
        .output()
        .expect("cwctl should start")
}

#[test]
fn help_and_version_succeed() {
    let data_dir = temporary_directory();
    let help = run(&["--help"], &data_dir);
    let version = run(&["--version"], &data_dir);

    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("plugin install"));
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout),
        "cwctl 27.0.0-alpha1\n"
    );
}

#[test]
fn invalid_command_returns_usage_error() {
    let data_dir = temporary_directory();
    let output = run(&["plugin", "unknown"], &data_dir);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown plugin command"));
}

#[test]
fn list_reports_empty_manifest() {
    let data_dir = temporary_directory();
    let output = run(&["plugin", "list"], &data_dir);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "No plugins installed.\n"
    );
    if data_dir.exists() {
        fs::remove_dir_all(data_dir).expect("temporary directory should be removed");
    }
}
