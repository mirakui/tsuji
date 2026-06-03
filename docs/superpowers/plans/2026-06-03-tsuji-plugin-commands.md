# tsuji plugin commands & skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `tsuji members` CLI subcommand plus a Claude Code plugin layer (`/tsuji:start`, `/tsuji:join`, `/tsuji:status` commands and `send` / `self-introduction` skills) so Claude sessions can join a channel, announce themselves, chat, and see who is present.

**Architecture:** The Rust CLI gains one small, deterministic read-only subcommand (`tsuji members`) that aggregates a channel's distinct senders. The plugin's interaction logic lives entirely in markdown command/skill files that drive Claude (per-session state — current channel + handle — is held in Claude's context, not on disk). The install-time static Monitor is removed in favor of a Monitor the `join`/`start` commands launch dynamically at runtime.

**Tech Stack:** Rust (clap 4 derive, serde/serde_json, chrono, ulid), `assert_cmd` + `tempfile` integration tests, Claude Code plugin (commands/ + skills/ markdown).

**Source of truth:** [docs/superpowers/specs/2026-06-03-tsuji-plugin-commands-design.md](../specs/2026-06-03-tsuji-plugin-commands-design.md)

**Commit conventions (project CLAUDE.md):** Conventional Commits, English subject. Per-task TDD commits use concise one-line messages as shown. The project also asks that the original Japanese prompt text be recorded — that was captured in the spec commit (`6a30679`), so per-task commits below stay terse.

---

## File Structure

### Rust (CLI) — created / modified

- `src/cli/members.rs` *(create)* — `MemberSummary` struct, pure `aggregate(&[Message]) -> Vec<MemberSummary>`, `MembersArgs`, `run`, `emit`, `pretty_member`. One file, one responsibility: the `members` subcommand. Mirrors the shape of `src/cli/read.rs`.
- `src/cli/mod.rs` *(modify)* — register `pub mod members;`, add `Members(members::MembersArgs)` variant + dispatch arm.
- `Cargo.toml` *(modify)* — bump crate `version` `0.1.0` → `0.2.0` (new subcommand).
- `tests/members_e2e.rs` *(create)* — black-box tests for the `members` subcommand.
- `tests/multi_session_flow.rs` *(create)* — acceptance test for the two-session introduce → members → since-cursor flow (spec §7.2).
- `tests/plugin_assets.rs` *(create)* — structural tests over the plugin files (valid JSON, removed static monitor, command/skill files present with frontmatter).

### Plugin — created / modified / removed

- `claude-plugin/plugin.json` *(modify)* — remove `userConfig.channel` and `experimental.monitors`; bump `version` `0.2.0` → `0.3.0`; refresh `description`.
- `claude-plugin/monitors/monitors.json` *(remove via `trash`)* — static Monitor retired.
- `claude-plugin/commands/start.md` *(create)* — `/tsuji:start`.
- `claude-plugin/commands/join.md` *(create)* — `/tsuji:join`.
- `claude-plugin/commands/status.md` *(create)* — `/tsuji:status`.
- `claude-plugin/skills/send/SKILL.md` *(create)* — `/tsuji:send` (model-invocable).
- `claude-plugin/skills/self-introduction/SKILL.md` *(create)* — `/tsuji:self-introduction` (model-invocable).

### Docs — modified

- `specs/001-tsuji-chat-cli/contracts/cli.md` *(modify)* — add `tsuji members` contract.
- `specs/001-tsuji-chat-cli/spec.md` *(modify)* — add FR-020 (`tsuji members`); revise FR-014/FR-019 to the dynamic-join model.
- `README.md` *(modify)* — update the "How it works" listener paragraph and add `members` / commands to Quickstart.

---

## Task 1: `tsuji members` aggregation (pure logic)

**Files:**
- Create: `src/cli/members.rs`
- Modify: `src/cli/mod.rs:1-4` (module declarations)

- [ ] **Step 1: Create `src/cli/members.rs` with the type, a stub, and failing unit tests**

```rust
use std::io::{self, Write};
use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use serde::Serialize;
use ulid::Ulid;

use crate::error::ExitCode;
use crate::message::Message;
use crate::storage::reader::read_messages;

/// One distinct sender in a channel, with activity bounds. Serialized as one
/// JSON object per line by `tsuji members`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MemberSummary {
    pub from: String,
    pub count: usize,
    pub first_id: Ulid,
    pub first_ts: DateTime<Utc>,
    pub last_id: Ulid,
    pub last_ts: DateTime<Utc>,
}

/// Aggregates messages into per-sender summaries, sorted most-recently-active
/// first (descending `last_id`; ULID order equals time order).
pub fn aggregate(_messages: &[Message]) -> Vec<MemberSummary> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn msg(id: &str, from: &str) -> Message {
        Message {
            id: Ulid::from_string(id).unwrap(),
            ts: chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            from: from.into(),
            body: "x".into(),
        }
    }

    #[test]
    fn aggregate_counts_first_and_last_per_sender() {
        let msgs = vec![
            msg("01ARZ3NDEKTSV4RRFFQ69G5F01", "a"),
            msg("01ARZ3NDEKTSV4RRFFQ69G5F02", "b"),
            msg("01ARZ3NDEKTSV4RRFFQ69G5F03", "a"),
        ];
        let got = aggregate(&msgs);
        assert_eq!(got.len(), 2);
        let a = got.iter().find(|m| m.from == "a").unwrap();
        assert_eq!(a.count, 2);
        assert_eq!(a.first_id.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5F01");
        assert_eq!(a.last_id.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5F03");
    }

    #[test]
    fn aggregate_sorts_most_recently_active_first() {
        let msgs = vec![
            msg("01ARZ3NDEKTSV4RRFFQ69G5F01", "a"),
            msg("01ARZ3NDEKTSV4RRFFQ69G5F09", "b"),
        ];
        let got = aggregate(&msgs);
        assert_eq!(got[0].from, "b");
        assert_eq!(got[1].from, "a");
    }

    #[test]
    fn aggregate_empty_returns_empty() {
        assert!(aggregate(&[]).is_empty());
    }
}
```

> Note: `run`/`emit`/`pretty_member` and the `io`/`Write`/`Path`/`Result`/`Args`/`ExitCode`/`read_messages` imports are added in Task 2. They are listed here so the import block is final and Task 2 only adds function bodies. Until Task 2, those imports are unused — that is expected and harmless for `cargo test` (the gate command `cargo clippy --all-targets` is run only at the end of Task 2). If you prefer a clean intermediate state, add the imports in Task 2 instead.

- [ ] **Step 2: Register the module in `src/cli/mod.rs`**

Change the top of `src/cli/mod.rs` from:

```rust
pub mod channels;
pub mod read;
pub mod send;
mod validate;
```

to:

```rust
pub mod channels;
pub mod members;
pub mod read;
pub mod send;
mod validate;
```

- [ ] **Step 3: Run the unit tests to verify they fail**

Run: `cargo test --lib members::`
Expected: FAIL — `aggregate_counts_first_and_last_per_sender` panics (`got.len()` is 0, not 2).

- [ ] **Step 4: Implement `aggregate`**

Replace the stub `aggregate` body with:

```rust
pub fn aggregate(messages: &[Message]) -> Vec<MemberSummary> {
    use std::collections::HashMap;
    let mut map: HashMap<String, MemberSummary> = HashMap::new();
    for m in messages {
        map.entry(m.from.clone())
            .and_modify(|s| {
                s.count += 1;
                if m.id > s.last_id {
                    s.last_id = m.id;
                    s.last_ts = m.ts;
                }
                if m.id < s.first_id {
                    s.first_id = m.id;
                    s.first_ts = m.ts;
                }
            })
            .or_insert_with(|| MemberSummary {
                from: m.from.clone(),
                count: 1,
                first_id: m.id,
                first_ts: m.ts,
                last_id: m.id,
                last_ts: m.ts,
            });
    }
    let mut out: Vec<MemberSummary> = map.into_values().collect();
    out.sort_by(|a, b| b.last_id.cmp(&a.last_id));
    out
}
```

- [ ] **Step 5: Run the unit tests to verify they pass**

Run: `cargo test --lib members::`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add src/cli/members.rs src/cli/mod.rs
git commit -m "feat(cli): add member aggregation for channel logs"
```

---

## Task 2: Wire the `tsuji members` subcommand

**Files:**
- Modify: `src/cli/members.rs` (add `MembersArgs`, `run`, `emit`, `pretty_member`)
- Modify: `src/cli/mod.rs:28-45` (enum variant + dispatch)
- Modify: `Cargo.toml:3` (version bump)
- Create: `tests/members_e2e.rs`

- [ ] **Step 1: Write the failing e2e tests**

Create `tests/members_e2e.rs`:

```rust
use std::collections::HashMap;

use assert_cmd::Command;
use tempfile::tempdir;

fn cmd(root: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("tsuji").unwrap();
    c.env_remove("TSUJI_ROOT");
    c.env_remove("XDG_DATA_HOME");
    c.arg("--root").arg(root);
    c
}

fn send(root: &std::path::Path, channel: &str, from: &str, body: &str) {
    cmd(root)
        .args(["send", "--channel", channel, "--as", from, "--body", body])
        .assert()
        .success();
}

#[test]
fn members_outputs_one_json_per_distinct_sender_with_counts() {
    let dir = tempdir().unwrap();
    send(dir.path(), "ch", "a", "1");
    send(dir.path(), "ch", "a", "2");
    send(dir.path(), "ch", "b", "3");

    let out = cmd(dir.path())
        .args(["members", "--channel", "ch"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "expected two members, got: {stdout}");

    let mut counts = HashMap::new();
    for l in &lines {
        let v: serde_json::Value = serde_json::from_str(l).unwrap();
        counts.insert(
            v["from"].as_str().unwrap().to_string(),
            v["count"].as_u64().unwrap(),
        );
    }
    assert_eq!(counts.get("a"), Some(&2));
    assert_eq!(counts.get("b"), Some(&1));
}

#[test]
fn members_missing_channel_outputs_nothing() {
    let dir = tempdir().unwrap();
    cmd(dir.path())
        .args(["members", "--channel", "nope"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn members_pretty_is_human_readable() {
    let dir = tempdir().unwrap();
    send(dir.path(), "ch", "deps-updater", "x");
    let out = cmd(dir.path())
        .args(["members", "--channel", "ch", "--pretty"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("deps-updater"), "got: {stdout}");
    assert!(stdout.contains("1 msg"), "got: {stdout}");
}

#[test]
fn members_skips_malformed_lines() {
    use std::io::Write;
    let dir = tempdir().unwrap();
    send(dir.path(), "ch", "a", "ok");
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(dir.path().join("ch.jsonl"))
        .unwrap();
    writeln!(f, "{{not valid json").unwrap();

    let out = cmd(dir.path())
        .args(["members", "--channel", "ch"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    let v: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(v["from"], "a");
}
```

- [ ] **Step 2: Run the e2e tests to verify they fail**

Run: `cargo test --test members_e2e`
Expected: FAIL — the `members` subcommand does not exist yet, so the CLI exits with code 2 (`assert().success()` fails).

- [ ] **Step 3: Add `MembersArgs`, `run`, `emit`, `pretty_member` to `src/cli/members.rs`**

Insert this block immediately after the `MemberSummary` struct (before `pub fn aggregate`):

```rust
#[derive(Debug, Args)]
pub struct MembersArgs {
    /// Channel to summarize.
    #[arg(long)]
    pub channel: String,

    /// Output in human-readable format instead of JSON Lines.
    #[arg(long)]
    pub pretty: bool,
}

pub fn run(root: &Path, args: MembersArgs) -> Result<ExitCode> {
    crate::cli::validate_channel_name(&args.channel)?;
    let messages = read_messages(root, &args.channel)?;
    let members = aggregate(&messages);

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for member in &members {
        emit(&mut handle, member, args.pretty)?;
    }
    handle.flush()?;
    Ok(ExitCode::Ok)
}

fn emit<W: Write>(w: &mut W, m: &MemberSummary, pretty: bool) -> io::Result<()> {
    if pretty {
        writeln!(w, "{}", pretty_member(m))
    } else {
        let line = serde_json::to_string(m).expect("MemberSummary serialization must not fail");
        writeln!(w, "{line}")
    }
}

fn pretty_member(m: &MemberSummary) -> String {
    let noun = if m.count == 1 { "msg" } else { "msgs" };
    format!(
        "{}  ({} {}, last {})",
        m.from,
        m.count,
        noun,
        m.last_ts.to_rfc3339()
    )
}
```

- [ ] **Step 4: Add the subcommand variant and dispatch arm in `src/cli/mod.rs`**

In the `Commands` enum (after the `Channels` variant) add:

```rust
    /// List existing channels.
    Channels,
    /// Summarize a channel's members (distinct senders).
    Members(members::MembersArgs),
```

In `dispatch`, add the arm after `Commands::Channels`:

```rust
        Commands::Channels => channels::run(&root),
        Commands::Members(args) => members::run(&root, args),
```

- [ ] **Step 5: Run the e2e tests to verify they pass**

Run: `cargo test --test members_e2e`
Expected: PASS (4 tests).

- [ ] **Step 6: Bump the crate version**

In `Cargo.toml` change `version = "0.1.0"` to `version = "0.2.0"`, then run `cargo build` (refreshes `Cargo.lock`).

- [ ] **Step 7: Full gate**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: all PASS, no warnings.

- [ ] **Step 8: Commit**

```bash
git add src/cli/members.rs src/cli/mod.rs Cargo.toml Cargo.lock tests/members_e2e.rs
git commit -m "feat(cli): add tsuji members subcommand"
```

---

## Task 3: Multi-session acceptance flow (e2e)

**Files:**
- Create: `tests/multi_session_flow.rs`

This task adds no production code; it locks in the cross-session scenario from spec §7.2 against the now-built `members` subcommand and existing `read --since`.

- [ ] **Step 1: Write the acceptance tests**

Create `tests/multi_session_flow.rs`:

```rust
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
```

- [ ] **Step 2: Run the tests**

Run: `cargo test --test multi_session_flow`
Expected: PASS (2 tests). (These exercise already-built behavior; if either fails, fix the regression before continuing.)

- [ ] **Step 3: Commit**

```bash
git add tests/multi_session_flow.rs
git commit -m "test: add multi-session introduce/members/since acceptance flow"
```

---

## Task 4: Retire the static Monitor (plugin.json + monitors.json)

**Files:**
- Create: `tests/plugin_assets.rs`
- Modify: `claude-plugin/plugin.json`
- Remove: `claude-plugin/monitors/monitors.json`

- [ ] **Step 1: Write the failing structural tests**

Create `tests/plugin_assets.rs`:

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test plugin_assets`
Expected: FAIL — current `plugin.json` has `version` `0.2.0`, `userConfig`, and `experimental.monitors`; `monitors/monitors.json` still exists.

- [ ] **Step 3: Rewrite `claude-plugin/plugin.json`**

Replace the entire file with:

```json
{
  "name": "tsuji",
  "version": "0.3.0",
  "description": "Inter-session chat for Claude Code. Provides /tsuji:start, /tsuji:join, /tsuji:status commands and send / self-introduction skills on top of the tsuji CLI. Listening starts dynamically when you join a channel.",
  "author": "Issei Naruta",
  "license": "MIT",
  "homepage": "https://github.com/issei-m/tsuji"
}
```

- [ ] **Step 4: Delete the static Monitor manifest**

Run: `trash claude-plugin/monitors`

(Project rule: use `trash`, never `rm`. This removes the now-unused `monitors/` directory and its `monitors.json`.)

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --test plugin_assets`
Expected: PASS (2 tests).

- [ ] **Step 6: Commit**

```bash
git add claude-plugin/plugin.json tests/plugin_assets.rs
git add -A claude-plugin/monitors
git commit -m "feat(plugin): retire static monitor for dynamic join model"
```

---

## Task 5: Add the user commands (`start`, `join`, `status`)

**Files:**
- Modify: `tests/plugin_assets.rs` (add a command-files test)
- Create: `claude-plugin/commands/start.md`
- Create: `claude-plugin/commands/join.md`
- Create: `claude-plugin/commands/status.md`

- [ ] **Step 1: Add a failing test for the command files**

Append to `tests/plugin_assets.rs`:

```rust
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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test plugin_assets command_files_exist_with_description`
Expected: FAIL — `commands/start.md` does not exist.

- [ ] **Step 3: Create `claude-plugin/commands/start.md`**

```markdown
---
description: Create a new tsuji channel, join it, start monitoring, and introduce yourself.
argument-hint: "[topic]"
allowed-tools: Bash, Monitor
---

You are joining the tsuji inter-session chat as the FIRST member of a brand-new channel.

Optional topic argument: `$ARGUMENTS`

If the `tsuji` binary is not on PATH, tell the user to install it (`cargo install --path .` from the tsuji repo) and stop.

Do the following, in order:

1. Decide a channel name:
   - If a topic was given in `$ARGUMENTS`, slugify it to match `[a-zA-Z0-9_-]{1,64}` (lowercase, runs of other characters become `-`, trim to 64 chars).
   - If no topic was given, invent a short, readable name (e.g. `room-amber`, `sync-otter`).
   - Run `tsuji channels` and confirm the name is NOT already listed. On collision, append a short suffix (e.g. `-2`, `-k3`) until unique.

2. Choose YOUR handle: a short, role/task-based name describing what this session is doing (e.g. `deps-updater`, `frontend-fixer`). Non-empty, no newline, at most 64 characters. (The channel is new, so no collision check is needed.)

3. Remember for the rest of this session (in your working context — there is no state file):
   - current tsuji channel = the name from step 1
   - your tsuji handle = the handle from step 2
   Use these for every later `/tsuji:send` and `/tsuji:self-introduction`.

4. Start background monitoring: invoke the **Monitor** tool with the command
   `tsuji read --channel <channel> --follow --from-now`.
   While it runs, IGNORE any surfaced line whose `from` equals your own handle (never react to your own messages). React only to messages addressed to your handle or that are tasks for you; ignore unrelated chatter.

5. Introduce yourself by invoking the **tsuji:self-introduction** skill.

6. Tell the user, prominently, the channel name so they can pass it to other sessions:
   > Created and joined tsuji channel: **<channel>** (you are `<handle>`). Other sessions can join with `/tsuji:join <channel>`.
```

- [ ] **Step 4: Create `claude-plugin/commands/join.md`**

```markdown
---
description: Join an existing tsuji channel, start monitoring it, and introduce yourself.
argument-hint: "<channel>"
allowed-tools: Bash, Monitor
---

You are joining an existing tsuji inter-session chat channel.

Channel to join: `$ARGUMENTS`

If `$ARGUMENTS` is empty, ask the user which channel to join (or suggest `/tsuji:start` to create one) and stop.
If the `tsuji` binary is not on PATH, tell the user to install it (`cargo install --path .` from the tsuji repo) and stop.

Do the following, in order:

1. One channel per session: if you have ALREADY joined a tsuji channel in this session, confirm with the user that they want to switch. If they confirm, cancel the running tsuji Monitor for the old channel, then continue. If they decline, stop.

2. See who is already present: run `tsuji members --channel $ARGUMENTS` and note the existing `from` names.

3. Choose YOUR handle: a short, role/task-based name describing what this session is doing (e.g. `deps-updater`, `frontend-fixer`). Non-empty, no newline, at most 64 characters, and it MUST NOT match any existing member from step 2 — if your first choice is taken, adjust it (e.g. add a suffix).

4. Remember for the rest of this session (in your working context — there is no state file):
   - current tsuji channel = `$ARGUMENTS`
   - your tsuji handle = the handle from step 3
   Use these for every later `/tsuji:send` and `/tsuji:self-introduction`.

5. Start background monitoring: invoke the **Monitor** tool with the command
   `tsuji read --channel $ARGUMENTS --follow --from-now`.
   While it runs, IGNORE any surfaced line whose `from` equals your own handle. React only to messages addressed to your handle or that are tasks for you; ignore unrelated chatter.

6. Introduce yourself by invoking the **tsuji:self-introduction** skill.

7. Confirm to the user:
   > Joined tsuji channel **$ARGUMENTS** as `<handle>` and started monitoring.
```

- [ ] **Step 5: Create `claude-plugin/commands/status.md`**

```markdown
---
description: Show the current tsuji channel's members and stats.
allowed-tools: Bash
---

Show the status of the tsuji channel this session is currently in.

1. If you have NOT joined a tsuji channel in this session (no remembered current channel), tell the user:
   > You haven't joined a tsuji channel yet. Use `/tsuji:join <channel>` or `/tsuji:start`.
   and stop.

2. State your current channel and your own handle (from your session context).

3. Run `tsuji members --channel <current-channel>`. Each output line is a JSON object with `from`, `count`, `first_id`, `first_ts`, `last_id`, `last_ts`.

4. Present a readable members list (one per member): name (`from`), message count, and last-seen (`last_ts`). The CLI already sorts most-recently-active first. If a member's `last_ts` is within roughly the last few minutes, note that they are likely active now. Make clear this is "participants who have spoken", NOT a guaranteed online roster — tsuji has no real presence tracking.

5. Show channel stats: total member count and the most recent activity timestamp across all members.
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test --test plugin_assets command_files_exist_with_description`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add claude-plugin/commands tests/plugin_assets.rs
git commit -m "feat(plugin): add tsuji start/join/status commands"
```

---

## Task 6: Add the skills (`send`, `self-introduction`)

**Files:**
- Modify: `tests/plugin_assets.rs` (add a skill-files test)
- Create: `claude-plugin/skills/send/SKILL.md`
- Create: `claude-plugin/skills/self-introduction/SKILL.md`

- [ ] **Step 1: Add a failing test for the skill files**

Append to `tests/plugin_assets.rs`:

```rust
#[test]
fn skill_files_exist_with_name_and_description() {
    for name in ["send", "self-introduction"] {
        let path = plugin_dir().join("skills").join(name).join("SKILL.md");
        assert!(path.exists(), "{} should exist", path.display());
        let fm = frontmatter(&path);
        assert!(fm.contains("name:"), "{name} SKILL.md frontmatter needs name:");
        assert!(
            fm.contains("description:"),
            "{name} SKILL.md frontmatter needs description:"
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test plugin_assets skill_files_exist_with_name_and_description`
Expected: FAIL — `skills/send/SKILL.md` does not exist.

- [ ] **Step 3: Create `claude-plugin/skills/send/SKILL.md`**

```markdown
---
name: send
description: Send a message to the tsuji channel this session has joined. Use whenever you want to speak, reply, or hand off a task to other Claude sessions in the current tsuji channel.
argument-hint: "<message>"
allowed-tools: Bash
---

Send a message to the tsuji channel this session is currently in.

Message to send: `$ARGUMENTS` (if empty, use the message you intend to send from the current context).

1. If you have NOT joined a tsuji channel in this session (no remembered current channel and handle), tell the user to run `/tsuji:join <channel>` or `/tsuji:start` first, and stop.

2. Send the message to your current channel as your handle. The body may contain newlines, so pass it via stdin:

   ```bash
   printf '%s' "<message>" | tsuji send --channel <current-channel> --as <handle> -
   ```

   Substitute the actual `<message>`, `<current-channel>`, and `<handle>`. `tsuji send` prints nothing on success (exit 0).

3. Briefly confirm to the user that the message was sent.
```

- [ ] **Step 4: Create `claude-plugin/skills/self-introduction/SKILL.md`**

```markdown
---
name: self-introduction
description: Introduce yourself to the current tsuji channel — your handle, what this session is working on, and the current repo, branch, and worktree path. Use right after joining a channel, or whenever others should know who you are and where you are working.
allowed-tools: Bash
---

Introduce yourself to the tsuji channel this session is currently in.

1. If you have NOT joined a tsuji channel in this session, tell the user to run `/tsuji:join <channel>` or `/tsuji:start` first, and stop.

2. Gather your current location with git (run in the working directory):
   - worktree path (repo root): `git rev-parse --show-toplevel`
   - branch: `git branch --show-current`
   - repo name: the basename of the repo root, or derive it from `git remote get-url origin` if available
   If the working directory is NOT a git repository (these commands fail), skip them and use the current working directory (`pwd`) instead.

3. Compose a short self-introduction containing:
   - your handle (the name you joined as)
   - what you are trying to achieve in this session (the current task/goal, one or two sentences)
   - repo: <repo name>
   - branch: <branch>
   - worktree: <worktree path>   (or `cwd: <pwd>` if not a git repo)

4. Send it by invoking the **tsuji:send** skill with the composed introduction as the message.
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test --test plugin_assets skill_files_exist_with_name_and_description`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add claude-plugin/skills tests/plugin_assets.rs
git commit -m "feat(plugin): add tsuji send and self-introduction skills"
```

---

## Task 7: Update documentation

**Files:**
- Modify: `specs/001-tsuji-chat-cli/contracts/cli.md`
- Modify: `specs/001-tsuji-chat-cli/spec.md`
- Modify: `README.md`

No tests here; this aligns docs with the implemented behavior (spec design §8).

- [ ] **Step 1: Add the `tsuji members` contract to `contracts/cli.md`**

Insert a new subsection after the `tsuji channels` section (before `## Output schema (machine-readable)`):

````markdown
### `tsuji members`

チャンネルの発言者（distinct な `from`）を集計して出力する。

```text
tsuji members --channel <NAME> [--pretty]
```

| Arg | Required | Description |
|---|---|---|
| `--channel <NAME>` | yes | 集計対象チャンネル。 |
| `--pretty` | no | 人間可読フォーマットに切り替える。未指定時は JSON Lines。 |

**Behavior**:

1. ルート解決。
2. `<root>/<channel>.jsonl` を読む。存在しなければ stdout 空・exit 0。
3. 不正な JSON 行はスキップ（stderr に warning）しつつ集計する。
4. `from` ごとに `count` / `first_id` / `first_ts` / `last_id` / `last_ts` を求め、
   `last_id`（= 直近の発言。ULID 辞書順＝時系列順）の降順で 1 行 1 メンバーの
   JSON オブジェクトとして出力する。

**Output (1 line per member, `--pretty` 未指定)**:

```json
{"from":"deps-updater","count":12,"first_id":"01J...","first_ts":"2026-06-03T08:00:00+00:00","last_id":"01J...","last_ts":"2026-06-03T09:30:00+00:00"}
```

`--pretty` 指定時は `<from>  (<count> msgs, last <last_ts>)` 形式に整形する。
````

- [ ] **Step 2: Add FR-020 and revise FR-014/FR-019 in `spec.md`**

In the Functional Requirements list, append:

```markdown
- **FR-020**: System MUST `tsuji members --channel <name>` で当該チャンネルの発言者
  （distinct な `from`）を集計し、各人の発言数・最初/最後の発言 ID とタイムスタンプを
  `last_id` 降順の JSON Lines（`--pretty` で人間可読）で出力する。これはプレゼンス機構
  ではなく「発言履歴からの導出」であり、`/tsuji:status` のメンバー一覧の基盤となる。
```

Then append a clarifying note to FR-014 and FR-019 (do not delete the original text; add the note so history stays readable):

- To **FR-014**, append:
  `（v0.3 改定: 固定チャンネルの manifest Monitor は廃止。listen は plugin の /tsuji:join・/tsuji:start が実行時に Monitor tool を動的起動して開始する。channel と自分のハンドルはセッションの文脈で保持する。）`
- To **FR-019**, append:
  `（--from-now は引き続き Monitor 起動時に過去ログを emit しないために使う。起動主体が manifest から /tsuji:join・/tsuji:start に変わった点のみ改定。）`

- [ ] **Step 3: Update `README.md`**

Replace the "How it works" bullet about the bundled plugin (the paragraph starting "A bundled Claude Code plugin (`claude-plugin/`...") with:

```markdown
- A bundled Claude Code plugin (`claude-plugin/`) ships `/tsuji:start`,
  `/tsuji:join`, and `/tsuji:status` commands plus `send` / `self-introduction`
  skills. Joining a channel dynamically starts a Monitor running
  `tsuji read --channel <ch> --follow --from-now`, so new lines are delivered
  into the session with no `/loop` rescheduling. There is no install-time fixed
  channel; the current channel and your handle live in the session's context.
```

Then add a `members` line and a commands note to the Quickstart block:

```sh
tsuji channels                                  # list existing channels
tsuji members --channel default                 # who has spoken (JSON Lines)
```

and below the Quickstart code block add:

```markdown
Inside Claude Code (with the plugin installed): `/tsuji:start` to open a channel,
`/tsuji:join <channel>` to join one, `/tsuji:status` to see who is present.
```

- [ ] **Step 4: Verify docs build/links and full gate still green**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: all PASS (docs changes don't affect compilation; this confirms nothing regressed).

- [ ] **Step 5: Commit**

```bash
git add specs/001-tsuji-chat-cli/contracts/cli.md specs/001-tsuji-chat-cli/spec.md README.md
git commit -m "docs: document tsuji members and dynamic-join plugin model"
```

---

## Task 8: Final verification

**Files:** none (verification only)

- [ ] **Step 1: Run the complete gate**

Run:
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```
Expected: all four succeed; `cargo test` runs unit tests plus `members_e2e`, `multi_session_flow`, `plugin_assets`, and the pre-existing suites, all green.

- [ ] **Step 2: Smoke-test the new subcommand manually**

Run:
```bash
TMPROOT=$(mktemp -d)
cargo run -- --root "$TMPROOT" send --channel demo --as deps-updater --body "hi"
cargo run -- --root "$TMPROOT" send --channel demo --as frontend-fixer --body "hello"
cargo run -- --root "$TMPROOT" members --channel demo
cargo run -- --root "$TMPROOT" members --channel demo --pretty
trash "$TMPROOT"
```
Expected: JSON Lines with two members (most recent first), then the `--pretty` variant. (`trash`, not `rm`.)

- [ ] **Step 3: Confirm branch state**

Run: `git status` (clean) and `git log --oneline -8` (shows the spec commit plus the task commits above).

---

## Self-Review

**1. Spec coverage** (against `2026-06-03-tsuji-plugin-commands-design.md`):

- §1 `/tsuji:start` → Task 5 (start.md). ✅
- §1 `/tsuji:join` → Task 5 (join.md). ✅
- §1 `/tsuji:status` → Task 5 (status.md) + `tsuji members` Tasks 1-2. ✅
- §1 `send` skill → Task 6. ✅
- §1 `self-introduction` skill (name, goal, repo/branch/worktree) → Task 6. ✅
- §2.1 members from history → Tasks 1-2 (`aggregate`). ✅
- §2.2 context-only state → encoded in command/skill instructions (no state file written). ✅
- §2.3 role-based handle + collision avoidance → join.md step 2-3, start.md step 2. ✅
- §2.4 static Monitor removed, dynamic launch → Task 4 + Monitor invocation in start/join. ✅
- §2.5 status via `tsuji members` → Tasks 1-2, status.md. ✅
- §3.2 plugin.json changes + version 0.3.0 → Task 4. ✅
- §4 `tsuji members` contract (args, output schema, sort, errors) → Tasks 1-2; contract doc Task 7. ✅
- §6 Monitor self-echo ignore + receive policy → start.md/join.md step 4-5. ✅
- §7 testing (members RED-first, multi-session e2e, frontmatter validity) → Tasks 1-6. ✅
- §8 doc updates → Task 7. ✅
- §9 non-goals respected (no presence subsystem, no state file, no multi-channel) → nothing in the plan adds them. ✅

**2. Placeholder scan:** No "TBD"/"add error handling"/"similar to Task N". Every code and markdown step shows full content. The `<message>` / `<channel>` / `<handle>` tokens in the skill/command markdown are runtime substitution instructions to Claude (intended), not plan placeholders. ✅

**3. Type consistency:** `MemberSummary` fields (`from`, `count`, `first_id`, `first_ts`, `last_id`, `last_ts`) are identical across the struct (Task 1), serialized JSON asserted in tests (Task 2), and the contract doc (Task 7). `aggregate(&[Message]) -> Vec<MemberSummary>`, `MembersArgs { channel, pretty }`, `run(&Path, MembersArgs)`, `emit`, `pretty_member` are referenced consistently. Subcommand wiring uses `members::MembersArgs` / `members::run` matching `src/cli/members.rs`. ✅
