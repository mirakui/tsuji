use std::fs;
use std::path::{Path, PathBuf};

fn plugin_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("claude")
        .join("marketplace")
        .join("plugins")
        .join("tsuji")
}

/// Returns the YAML frontmatter block (the text between the first two `---`).
fn frontmatter(path: &Path) -> String {
    let raw = fs::read_to_string(path).unwrap();
    let mut parts = raw.splitn(3, "---");
    let _before = parts.next();
    parts
        .next()
        .unwrap_or_else(|| panic!("{} is missing a frontmatter block", path.display()))
        .to_string()
}

#[test]
fn command_files_exist_with_description() {
    for name in ["start", "join", "status"] {
        let path = plugin_dir().join("commands").join(format!("{name}.md"));
        assert!(path.exists(), "{} should exist", path.display());
        let fm = frontmatter(&path);
        assert!(
            fm.contains("description:"),
            "{name}.md frontmatter needs description:"
        );
    }
}

#[test]
fn plugin_json_is_valid_and_has_no_static_monitor() {
    let raw = fs::read_to_string(plugin_dir().join(".claude-plugin").join("plugin.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).expect("plugin.json must be valid JSON");
    assert_eq!(v["name"], "tsuji");
    assert_eq!(v["version"], "0.3.0");
    assert!(
        v.get("experimental").is_none(),
        "experimental.monitors must be removed"
    );
    assert!(
        v.get("userConfig").is_none(),
        "userConfig.channel must be removed"
    );
}

#[test]
fn static_monitor_manifest_is_removed() {
    assert!(
        !plugin_dir().join("monitors").join("monitors.json").exists(),
        "monitors/monitors.json should be deleted"
    );
}

#[test]
fn skill_files_exist_with_name_and_description() {
    for name in ["send", "self-introduction"] {
        let path = plugin_dir().join("skills").join(name).join("SKILL.md");
        assert!(path.exists(), "{} should exist", path.display());
        let fm = frontmatter(&path);
        assert!(
            fm.contains("name:"),
            "{name} SKILL.md frontmatter needs name:"
        );
        assert!(
            fm.contains("description:"),
            "{name} SKILL.md frontmatter needs description:"
        );
    }
}

#[test]
fn send_skill_reads_body_from_stdin_via_body_dash_flag() {
    // `tsuji send` reads the body from stdin only with `--body -`; a bare trailing
    // `-` is rejected by clap (exit 2). Guard against regressing to that broken form.
    let path = plugin_dir().join("skills").join("send").join("SKILL.md");
    let body = fs::read_to_string(&path).unwrap();
    assert!(
        body.contains("--body -"),
        "send SKILL.md must tell tsuji to read the body from stdin via `--body -`"
    );
    assert!(
        !body.contains("--as <handle> -\n"),
        "send SKILL.md must NOT use a bare trailing `-` (rejected by clap with exit 2)"
    );
}

#[test]
fn join_and_start_commands_require_persistent_monitor() {
    // Without `persistent: true` the Monitor times out (~5 min) and listening
    // silently stops, so both join and start must require it.
    for name in ["join", "start"] {
        let path = plugin_dir().join("commands").join(format!("{name}.md"));
        let body = fs::read_to_string(&path).unwrap();
        assert!(
            body.contains("persistent: true"),
            "{name}.md must require `persistent: true` for the background Monitor"
        );
    }
}

#[test]
fn join_and_start_monitors_exclude_current_handle() {
    for name in ["join", "start"] {
        let path = plugin_dir().join("commands").join(format!("{name}.md"));
        let body = fs::read_to_string(&path).unwrap();
        assert!(
            body.contains("--exclude-from"),
            "{name}.md Monitor command must exclude this session's handle"
        );
        assert!(
            body.contains("including your own"),
            "{name}.md must document that omitting --exclude-from reads all messages"
        );
    }
}
