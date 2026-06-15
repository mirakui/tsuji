use std::fs;
use std::path::{Path, PathBuf};

fn plugin_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("claude")
        .join("marketplace")
        .join("plugins")
        .join("tsuji")
}

fn codex_skills_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("codex")
        .join("skills")
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
fn codex_skill_files_exist_with_ui_metadata() {
    for name in [
        "tsuji-start",
        "tsuji-join",
        "tsuji-status",
        "tsuji-send",
        "tsuji-self-introduction",
    ] {
        let skill_path = codex_skills_dir().join(name).join("SKILL.md");
        assert!(skill_path.exists(), "{} should exist", skill_path.display());
        let fm = frontmatter(&skill_path);
        assert!(
            fm.contains("name:"),
            "{name} SKILL.md frontmatter needs name:"
        );
        assert!(
            fm.contains("description:"),
            "{name} SKILL.md frontmatter needs description:"
        );

        let agent_path = codex_skills_dir()
            .join(name)
            .join("agents")
            .join("openai.yaml");
        assert!(agent_path.exists(), "{} should exist", agent_path.display());
        let agent = fs::read_to_string(&agent_path).unwrap();
        assert!(
            agent.contains("display_name:"),
            "{name} openai.yaml should expose a display name"
        );
        assert!(
            agent.contains("default_prompt:"),
            "{name} openai.yaml should expose default usage guidance"
        );
    }
}

#[test]
fn codex_skill_usage_readme_lists_available_workflows() {
    let path = codex_skills_dir().join("README.md");
    assert!(path.exists(), "{} should exist", path.display());
    let body = fs::read_to_string(path).unwrap();
    for name in [
        "tsuji-start",
        "tsuji-join",
        "tsuji-status",
        "tsuji-send",
        "tsuji-self-introduction",
    ] {
        assert!(
            body.contains(name),
            "Codex skills README should list {name}"
        );
    }
}

#[test]
fn codex_send_skill_reads_body_from_stdin_via_body_dash_flag() {
    let path = codex_skills_dir().join("tsuji-send").join("SKILL.md");
    let body = fs::read_to_string(&path).unwrap();
    assert!(
        body.contains("--body -"),
        "tsuji-send must tell tsuji to read message bodies via `--body -`"
    );
    assert!(
        !body.contains("--as <handle> -\n"),
        "tsuji-send must NOT use a bare trailing `-`"
    );
}

#[test]
fn codex_join_and_start_skills_require_persistent_monitor() {
    for name in ["tsuji-join", "tsuji-start"] {
        let path = codex_skills_dir().join(name).join("SKILL.md");
        let body = fs::read_to_string(&path).unwrap();
        assert!(
            body.contains("persistent: true"),
            "{name} must require `persistent: true` for background monitoring"
        );
        assert!(
            body.contains("tsuji-self-introduction"),
            "{name} should introduce the session after monitoring starts"
        );
    }
}
