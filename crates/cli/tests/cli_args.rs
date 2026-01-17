use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sisyphus"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("AI Agent CLI"));
}

#[test]
fn test_cli_tui_command() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sisyphus"));
    cmd.arg("tui")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Start a TUI session"));
}

#[test]
fn test_cli_repl_command() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sisyphus"));
    cmd.arg("repl")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Start a REPL session"));
}

#[test]
fn test_cli_msg_command() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sisyphus"));
    cmd.arg("msg")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Send a one-shot message"));
}

#[test]
fn test_cli_serve_command() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sisyphus"));
    cmd.arg("serve")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Start a server"));
}

#[test]
fn test_cli_attach_flag() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sisyphus"));
    cmd.arg("repl")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("--attach"));
}
