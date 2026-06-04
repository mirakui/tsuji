use std::collections::HashMap;

use assert_cmd::Command;
use tempfile::tempdir;

fn cmd(root: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("tsuji").unwrap();
    c.env_remove("TSUJI_ROOT");
    c.env_remove("XDG_DATA_HOME");
    c.arg("--root").arg(root);
    c
}

fn send(root: &std::path::Path, channel: &str, from: &str, body: &str) {
    cmd(root)
        .args(["send", "--channel", channel, "--as", from, "--body", body])
        .assert()
        .success();
}

#[test]
fn members_outputs_one_json_per_distinct_sender_with_counts() {
    let dir = tempdir().unwrap();
    send(dir.path(), "ch", "a", "1");
    send(dir.path(), "ch", "a", "2");
    send(dir.path(), "ch", "b", "3");

    let out = cmd(dir.path())
        .args(["members", "--channel", "ch"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "expected two members, got: {stdout}");

    let mut counts = HashMap::new();
    for l in &lines {
        let v: serde_json::Value = serde_json::from_str(l).unwrap();
        counts.insert(
            v["from"].as_str().unwrap().to_string(),
            v["count"].as_u64().unwrap(),
        );
    }
    assert_eq!(counts.get("a"), Some(&2));
    assert_eq!(counts.get("b"), Some(&1));
}

#[test]
fn members_missing_channel_outputs_nothing() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args(["members", "--channel", "nope"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn members_pretty_is_human_readable() {
    let dir = tempdir().unwrap();
    send(dir.path(), "ch", "deps-updater", "x");
    let out = cmd(dir.path())
        .args(["members", "--channel", "ch", "--pretty"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("deps-updater"), "got: {stdout}");
    assert!(stdout.contains("1 msg"), "got: {stdout}");
}

#[test]
fn members_skips_malformed_lines() {
    use std::io::Write;
    let dir = tempdir().unwrap();
    send(dir.path(), "ch", "a", "ok");
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(dir.path().join("ch.jsonl"))
        .unwrap();
    writeln!(f, "{{not valid json").unwrap();

    let out = cmd(dir.path())
        .args(["members", "--channel", "ch"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    let v: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(v["from"], "a");
}

#[test]
fn members_rejects_invalid_channel_name() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args(["members", "--channel", "bad name!"])
        .assert()
        .failure();
}
