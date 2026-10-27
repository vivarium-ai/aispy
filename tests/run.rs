use std::process::Command;

#[test]
fn aispy_help_succeeds() {
    let status = Command::new(env!("CARGO_BIN_EXE_aispy"))
        .arg("--help")
        .status()
        .expect("failed to execute aispy");

    assert!(status.success());
}
