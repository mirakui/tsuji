use std::collections::HashSet;
use std::process::Command;
use std::thread;

use serde_json::Value;
use tempfile::tempdir;

fn send_one(exe: &std::path::Path, root: &std::path::Path, sender: &str, body: &str) {
    let status = Command::new(exe)
        .env_remove("TSUJI_ROOT")
        .env_remove("XDG_DATA_HOME")
        .arg("--root")
        .arg(root)
        .args([
            "send",
            "--channel",
            "stress",
            "--as",
            sender,
            "--body",
            body,
        ])
        .status()
        .unwrap();
    assert!(status.success(), "send {sender} {body} failed: {status}");
}

#[test]
fn one_hundred_concurrent_sends_produce_one_hundred_well_formed_unique_lines() {
    let dir = tempdir().unwrap();
    let exe = assert_cmd::cargo::cargo_bin("tsuji");
    let root = dir.path().to_path_buf();
    let exe2 = exe.clone();
    let root2 = root.clone();

    let h1 = thread::spawn(move || {
        for i in 0..50 {
            send_one(&exe, &root, "session-a", &format!("msg-a-{i}"));
        }
    });
    let h2 = thread::spawn(move || {
        for i in 0..50 {
            send_one(&exe2, &root2, "session-b", &format!("msg-b-{i}"));
        }
    });
    h1.join().unwrap();
    h2.join().unwrap();

    let content = std::fs::read_to_string(dir.path().join("stress.jsonl")).unwrap();
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 100, "expected 100 lines, got {}", lines.len());

    let mut ids: HashSet<String> = HashSet::new();
    for (i, line) in lines.iter().enumerate() {
        let v: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {i} not JSON: {line:?}: {e}"));
        let id = v
            .get("id")
            .and_then(Value::as_str)
            .expect("id field")
            .to_string();
        assert_eq!(id.len(), 26, "id should be 26 chars: {id}");
        assert!(ids.insert(id.clone()), "duplicate id {id} at line {i}");
        assert!(v.get("from").and_then(Value::as_str).is_some());
        assert!(v.get("body").and_then(Value::as_str).is_some());
    }
}
