use assert_cmd::Command;
use jsonschema::JSONSchema;
use tempfile::tempdir;

fn cmd(root: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("tsuji").unwrap();
    c.env_remove("TSUJI_ROOT");
    c.env_remove("XDG_DATA_HOME");
    c.arg("--root").arg(root);
    c
}

#[test]
fn every_read_line_matches_jsonl_schema() {
    let dir = tempdir().unwrap();
    for i in 0..3 {
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

    let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("specs/001-tsuji-chat-cli/contracts/jsonl-schema.json");
    let schema_str = std::fs::read_to_string(&schema_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", schema_path.display()));
    let schema_value: serde_json::Value = serde_json::from_str(&schema_str).unwrap();
    let validator = JSONSchema::compile(&schema_value).expect("schema compiles");

    let out = cmd(dir.path())
        .args(["read", "--channel", "default"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    for (i, line) in stdout.lines().enumerate() {
        let v: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {i} is not JSON: {line:?}: {e}"));
        if !validator.is_valid(&v) {
            let errors: Vec<String> = validator
                .validate(&v)
                .err()
                .into_iter()
                .flatten()
                .map(|e| e.to_string())
                .collect();
            panic!("line {i} fails schema: {v}; errors: {errors:?}");
        }
    }
}
