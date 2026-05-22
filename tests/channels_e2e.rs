use assert_cmd::Command;
use tempfile::tempdir;

fn cmd(root: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("tsuji").unwrap();
    c.env_remove("TSUJI_ROOT");
    c.env_remove("XDG_DATA_HOME");
    c.arg("--root").arg(root);
    c
}

#[test]
fn send_to_a_new_channel_auto_creates_it() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "newtopic",
            "--as",
            "agent-a",
            "--body",
            "hello",
        ])
        .assert()
        .success();
    assert!(
        dir.path().join("newtopic.jsonl").exists(),
        "channel file should be created"
    );
}

#[test]
fn channels_lists_existing_channel_names_alphabetically() {
    let dir = tempdir().unwrap();
    for name in ["zeta", "alpha", "beta"] {
        cmd(dir.path())
            .args(["send", "--channel", name, "--as", "agent-a", "--body", "x"])
            .assert()
            .success();
    }
    let out = cmd(dir.path()).arg("channels").assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["alpha", "beta", "zeta"]);
}

#[test]
fn cross_channel_reads_do_not_bleed() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args(["send", "--channel", "a", "--as", "agent", "--body", "in-a"])
        .assert()
        .success();
    cmd(dir.path())
        .args(["send", "--channel", "b", "--as", "agent", "--body", "in-b"])
        .assert()
        .success();

    let out_a = cmd(dir.path())
        .args(["read", "--channel", "a"])
        .assert()
        .success();
    let stdout_a = String::from_utf8(out_a.get_output().stdout.clone()).unwrap();
    assert!(stdout_a.contains("in-a"));
    assert!(!stdout_a.contains("in-b"));

    let out_b = cmd(dir.path())
        .args(["read", "--channel", "b"])
        .assert()
        .success();
    let stdout_b = String::from_utf8(out_b.get_output().stdout.clone()).unwrap();
    assert!(stdout_b.contains("in-b"));
    assert!(!stdout_b.contains("in-a"));
}

#[test]
fn channels_command_on_empty_root_outputs_nothing() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .arg("channels")
        .assert()
        .success()
        .stdout("");
}
