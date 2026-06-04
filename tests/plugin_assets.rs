use std::fs;
use std::path::{Path, PathBuf};

fn plugin_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("claude-plugin")
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
    let raw = fs::read_to_string(plugin_dir().join("plugin.json")).unwrap();
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
