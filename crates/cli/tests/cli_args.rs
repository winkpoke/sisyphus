use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("sisyphus").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("AI Agent CLI"));
}

#[test]
fn test_cli_chat_tui_flag() {
    let mut cmd = Command::cargo_bin("sisyphus").unwrap();
    cmd.arg("chat")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("--tui"));
}
