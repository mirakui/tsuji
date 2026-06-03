use std::fs;
use std::path::{Path, PathBuf};

fn plugin_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("claude-plugin")
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
