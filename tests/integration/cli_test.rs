use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("bondcli"));
}

#[test]
fn test_cli_create_help() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Create a new bond interface"));
}

#[test]
fn test_cli_list_help() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List all bond interfaces"));
}

#[test]
fn test_cli_add_help() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("add")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Add a slave interface"));
}

#[test]
fn test_cli_remove_help() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("remove")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Remove a slave interface"));
}

#[test]
fn test_cli_delete_help() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("delete")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Delete a bond interface"));
}

#[test]
fn test_cli_info_help() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("info")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show detailed information"));
}

#[test]
fn test_cli_status_help() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("status")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show status of all bonds"));
}

#[test]
fn test_cli_create_requires_slaves() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("create")
        .arg("--name")
        .arg("bond0")
        .assert()
        .failure();
}

#[test]
fn test_cli_create_with_slaves() {
    // This would fail on systems without eth0/eth1, but tests argument parsing
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("create")
        .arg("--name")
        .arg("bond0")
        .arg("--slaves")
        .arg("eth0")
        .arg("eth1")
        .arg("--dry-run")
        .assert()
        .failure(); // Should fail because eth0/eth1 don't exist
}

#[test]
fn test_cli_add_requires_bond_and_slave() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("add")
        .assert()
        .failure();
}

#[test]
fn test_cli_info_requires_bond() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("info")
        .assert()
        .failure();
}

#[test]
fn test_cli_delete_requires_bond() {
    let mut cmd = Command::cargo_bin("bondcli").unwrap();
    cmd.arg("delete")
        .assert()
        .failure();
}
