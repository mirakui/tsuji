use assert_cmd::Command;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::contains;
use serde_json::Value;
use tempfile::tempdir;

fn cmd(root: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("tsuji").unwrap();
    c.env_remove("TSUJI_ROOT");
    c.env_remove("XDG_DATA_HOME");
    c.arg("--root").arg(root);
    c
}

fn read_all_ids(root: &std::path::Path, channel: &str) -> Vec<String> {
    let out = cmd(root)
        .args(["read", "--channel", channel])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    stdout
        .lines()
        .map(|l| {
            serde_json::from_str::<Value>(l)
                .unwrap()
                .get("id")
                .and_then(Value::as_str)
                .unwrap()
                .to_string()
        })
        .collect()
}

#[test]
fn since_returns_only_messages_after_the_cursor() {
    let dir = tempdir().unwrap();
    for i in 0..5 {
        cmd(dir.path())
            .args([
                "send",
                "--channel",
                "x",
                "--as",
                "agent",
                "--body",
                &format!("m{i}"),
            ])
            .assert()
            .success();
    }
    let ids = read_all_ids(dir.path(), "x");
    let cursor = &ids[2];

    let out = cmd(dir.path())
        .args(["read", "--channel", "x", "--since", cursor])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let got_ids: Vec<String> = stdout
        .lines()
        .map(|l| {
            serde_json::from_str::<Value>(l)
                .unwrap()
                .get("id")
                .and_then(Value::as_str)
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(got_ids, vec![ids[3].clone(), ids[4].clone()]);
}

#[test]
fn since_with_ulid_greater_than_all_messages_returns_empty() {
    let dir = tempdir().unwrap();
    for i in 0..3 {
        cmd(dir.path())
            .args([
                "send",
                "--channel",
                "x",
                "--as",
                "agent",
                "--body",
                &format!("m{i}"),
            ])
            .assert()
            .success();
    }
    // ULID max is all Zs (Crockford)
    cmd(dir.path())
        .args([
            "read",
            "--channel",
            "x",
            "--since",
            "7ZZZZZZZZZZZZZZZZZZZZZZZZZ",
        ])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn malformed_since_fails_with_exit_1() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args(["read", "--channel", "x", "--since", "not-a-ulid"])
        .assert()
        .failure()
        .stderr(contains("ULID").or(contains("ulid")));
}
