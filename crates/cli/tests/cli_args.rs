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
fn test_cli_chat_tui_flag() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sisyphus"));
    cmd.arg("chat")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("--tui"));
}
