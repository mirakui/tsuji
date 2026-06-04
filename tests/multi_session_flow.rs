use assert_cmd::Command;
use tempfile::tempdir;

fn cmd(root: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("tsuji").unwrap();
    c.env_remove("TSUJI_ROOT");
    c.env_remove("XDG_DATA_HOME");
    c.arg("--root").arg(root);
    c
}

fn send(root: &std::path::Path, from: &str, body: &str) {
    cmd(root)
        .args(["send", "--channel", "room", "--as", from, "--body", body])
        .assert()
        .success();
}

#[test]
fn two_sessions_introduce_and_members_lists_both() {
    let dir = tempdir().unwrap();
    send(dir.path(), "deps-updater", "hi, I update deps");
    send(dir.path(), "frontend-fixer", "hi, I fix frontend");

    let out = cmd(dir.path())
        .args(["members", "--channel", "room"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let froms: Vec<String> = stdout
        .lines()
        .map(|l| {
            serde_json::from_str::<serde_json::Value>(l).unwrap()["from"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(froms.len(), 2, "got: {stdout}");
    assert!(froms.contains(&"deps-updater".to_string()));
    assert!(froms.contains(&"frontend-fixer".to_string()));
}

#[test]
fn since_cursor_returns_only_messages_after_the_cursor() {
    let dir = tempdir().unwrap();
    send(dir.path(), "a", "first");

    let out = cmd(dir.path())
        .args(["read", "--channel", "room"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let last_line = stdout.lines().last().unwrap();
    let last_id = serde_json::from_str::<serde_json::Value>(last_line).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    send(dir.path(), "b", "second");

    let out2 = cmd(dir.path())
        .args(["read", "--channel", "room", "--since", &last_id])
        .assert()
        .success();
    let s2 = String::from_utf8(out2.get_output().stdout.clone()).unwrap();
    assert!(s2.contains("second"), "got: {s2}");
    assert!(!s2.contains("first"), "got: {s2}");
}
