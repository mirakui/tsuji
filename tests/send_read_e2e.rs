use assert_cmd::Command;
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

#[test]
fn send_then_read_round_trips_a_message() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-a",
            "--body",
            "hello",
        ])
        .assert()
        .success()
        .stdout("");

    let out = cmd(dir.path())
        .args(["read", "--channel", "default"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line, got {stdout:?}");

    let v: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(v.get("from").and_then(Value::as_str), Some("agent-a"));
    assert_eq!(v.get("body").and_then(Value::as_str), Some("hello"));
    assert_eq!(v.get("id").and_then(Value::as_str).map(str::len), Some(26));
}

#[test]
fn read_against_missing_channel_returns_empty_with_success() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args(["read", "--channel", "nope"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn body_with_newlines_is_preserved() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-a",
            "--body",
            "line1\nline2",
        ])
        .assert()
        .success();

    let out = cmd(dir.path())
        .args(["read", "--channel", "default"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "newline must NOT break JSON Lines structure: {stdout:?}"
    );

    let v: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(v.get("body").and_then(Value::as_str), Some("line1\nline2"));
}

#[test]
fn two_consecutive_sends_produce_strictly_increasing_ulids() {
    let dir = tempdir().unwrap();
    for i in 0..5 {
        cmd(dir.path())
            .args([
                "send",
                "--channel",
                "default",
                "--as",
                "agent-a",
                "--body",
                &format!("msg {i}"),
            ])
            .assert()
            .success();
    }

    let out = cmd(dir.path())
        .args(["read", "--channel", "default"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let ids: Vec<String> = stdout
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
    assert_eq!(ids.len(), 5);
    for w in ids.windows(2) {
        assert!(
            w[0] < w[1],
            "ULIDs not strictly increasing: {} >= {}",
            w[0],
            w[1]
        );
    }
}

#[test]
fn empty_body_is_rejected() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-a",
            "--body",
            "",
        ])
        .assert()
        .failure()
        .stderr(contains("body"));
}

#[test]
fn body_dash_reads_message_from_stdin() {
    // The plugin's send skill pipes the body on stdin via `--body -`; this is the
    // contract that behavior depends on.
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-a",
            "--body",
            "-",
        ])
        .write_stdin("piped body\n")
        .assert()
        .success();

    let out = cmd(dir.path())
        .args(["read", "--channel", "default"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    // read_stdin trims a single trailing newline that shells/pipes add.
    assert_eq!(v.get("body").and_then(Value::as_str), Some("piped body"));
}

#[test]
fn bare_trailing_dash_is_rejected_as_arg_error() {
    // A bare positional `-` is NOT a valid way to read stdin; clap rejects it with
    // exit code 2. This is why the send skill must use `--body -` instead.
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args(["send", "--channel", "default", "--as", "agent-a", "-"])
        .write_stdin("ignored\n")
        .assert()
        .code(2);
}

#[test]
fn invalid_channel_name_is_rejected() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "has space",
            "--as",
            "agent-a",
            "--body",
            "x",
        ])
        .assert()
        .failure()
        .stderr(contains("channel"));
}
