use std::io::Read;
use std::process::{Command as StdCommand, Stdio};
use std::time::{Duration, Instant};

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
fn pretty_format_renders_single_line_inline() {
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
        .success();
    let out = cmd(dir.path())
        .args(["read", "--channel", "default", "--pretty"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    let l = lines[0];
    assert!(l.starts_with('['), "expected leading [ts]: {l}");
    assert!(
        l.ends_with(": hello") || l.contains(" agent-a: hello"),
        "got: {l}"
    );
}

#[test]
fn pretty_format_indents_continuation_lines() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-a",
            "--body",
            "line1\nline2\nline3",
        ])
        .assert()
        .success();
    let out = cmd(dir.path())
        .args(["read", "--channel", "default", "--pretty"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].ends_with(": line1"));
    assert_eq!(lines[1], "    line2");
    assert_eq!(lines[2], "    line3");
}

#[test]
fn follow_surfaces_new_messages_within_two_seconds() {
    let dir = tempdir().unwrap();
    // seed one so the file exists
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-a",
            "--body",
            "seed",
        ])
        .assert()
        .success();

    let exe = assert_cmd::cargo::cargo_bin("tsuji");
    let mut child = StdCommand::new(&exe)
        .env_remove("TSUJI_ROOT")
        .env_remove("XDG_DATA_HOME")
        .arg("--root")
        .arg(dir.path())
        .args(["read", "--channel", "default", "--follow"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Let the child read the seed first.
    std::thread::sleep(Duration::from_millis(600));

    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-b",
            "--body",
            "fresh",
        ])
        .assert()
        .success();

    let stdout = child.stdout.take().unwrap();
    let mut reader = stdout;
    let mut buf = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut got = String::new();
    while Instant::now() < deadline {
        let mut chunk = [0u8; 1024];
        // Non-blocking read by polling: set a short timeout via attempting a read after sleep.
        std::thread::sleep(Duration::from_millis(100));
        match reader.read(&mut chunk) {
            Ok(0) => continue,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                got = String::from_utf8_lossy(&buf).to_string();
                if got.contains("fresh") {
                    break;
                }
            }
            Err(_) => continue,
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    assert!(
        got.contains("fresh"),
        "follow did not surface the new message within 2s; got: {got:?}"
    );
}

#[test]
fn follow_with_from_now_skips_existing_messages() {
    let dir = tempdir().unwrap();
    // Seed two messages BEFORE the listener starts. With --from-now these
    // must not appear on the listener's stdout.
    for body in ["seed-1", "seed-2"] {
        cmd(dir.path())
            .args([
                "send",
                "--channel",
                "default",
                "--as",
                "agent-a",
                "--body",
                body,
            ])
            .assert()
            .success();
    }

    let exe = assert_cmd::cargo::cargo_bin("tsuji");
    let mut child = StdCommand::new(&exe)
        .env_remove("TSUJI_ROOT")
        .env_remove("XDG_DATA_HOME")
        .arg("--root")
        .arg(dir.path())
        .args(["read", "--channel", "default", "--follow", "--from-now"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Give the listener time to initialize and consume the existing tail.
    std::thread::sleep(Duration::from_millis(600));

    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-b",
            "--body",
            "fresh-after",
        ])
        .assert()
        .success();

    let mut reader = child.stdout.take().unwrap();
    let mut buf = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut got = String::new();
    while Instant::now() < deadline {
        let mut chunk = [0u8; 1024];
        std::thread::sleep(Duration::from_millis(100));
        match reader.read(&mut chunk) {
            Ok(0) => continue,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                got = String::from_utf8_lossy(&buf).to_string();
                if got.contains("fresh-after") {
                    break;
                }
            }
            Err(_) => continue,
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    assert!(
        got.contains("fresh-after"),
        "follow --from-now did not surface the new message within 2s; got: {got:?}"
    );
    assert!(
        !got.contains("seed-1") && !got.contains("seed-2"),
        "follow --from-now leaked existing messages; got: {got:?}"
    );
}

#[test]
fn follow_exclude_from_skips_own_messages_but_surfaces_others() {
    let dir = tempdir().unwrap();
    let exe = assert_cmd::cargo::cargo_bin("tsuji");
    let mut child = StdCommand::new(&exe)
        .env_remove("TSUJI_ROOT")
        .env_remove("XDG_DATA_HOME")
        .arg("--root")
        .arg(dir.path())
        .args([
            "read",
            "--channel",
            "default",
            "--follow",
            "--from-now",
            "--exclude-from",
            "agent-a",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    std::thread::sleep(Duration::from_millis(600));

    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-a",
            "--body",
            "own-after",
        ])
        .assert()
        .success();
    cmd(dir.path())
        .args([
            "send",
            "--channel",
            "default",
            "--as",
            "agent-b",
            "--body",
            "other-after",
        ])
        .assert()
        .success();

    let mut reader = child.stdout.take().unwrap();
    let mut buf = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut got = String::new();
    while Instant::now() < deadline {
        let mut chunk = [0u8; 1024];
        std::thread::sleep(Duration::from_millis(100));
        match reader.read(&mut chunk) {
            Ok(0) => continue,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                got = String::from_utf8_lossy(&buf).to_string();
                if got.contains("other-after") {
                    break;
                }
            }
            Err(_) => continue,
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    assert!(
        got.contains("other-after"),
        "follow --exclude-from did not surface the other message within 2s; got: {got:?}"
    );
    assert!(
        !got.contains("own-after"),
        "follow --exclude-from surfaced the excluded sender; got: {got:?}"
    );
}

#[test]
fn from_now_requires_follow() {
    let dir = tempdir().unwrap();
    // Without --follow, --from-now is meaningless and must be rejected by clap
    // (it would emit nothing otherwise). Expect exit code 2 (argument syntax).
    cmd(dir.path())
        .args(["read", "--channel", "default", "--from-now"])
        .assert()
        .failure();
}
