---

description: "Task list for 001-tsuji-chat-cli implementation"
---

# Tasks: tsuji — Inter-Session Chat CLI for Claude Code

**Input**: Design documents from `/specs/001-tsuji-chat-cli/`

**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/ ✓

**Tests**: TDD is mandatory per CLAUDE.md (t_wada style). Each user story phase writes failing tests first, then implements until green.

**Organization**: Tasks are grouped by user story so each story is independently testable and shippable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Maps to user stories in spec.md (US1 / US2 / US3 / US4)
- Each task lists exact file paths

## Path Conventions

Single Rust crate at repository root: `src/`, `tests/`, `claude-plugin/` per plan.md "Project Structure".

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Cargo プロジェクト初期化と依存 / ツール設定。

- [X] T001 Initialize Rust binary crate with `cargo init --name tsuji --bin` at repository root; verify resulting `Cargo.toml` and `src/main.rs` placeholder
- [X] T002 Add runtime dependencies (`clap` with `derive` feature, `ulid` with `serde` feature, `serde`, `serde_json`, `chrono` with `serde` feature, `fs2`, `anyhow`) under `[dependencies]` in Cargo.toml
- [X] T003 [P] Add dev-dependencies (`assert_cmd`, `predicates`, `tempfile`) under `[dev-dependencies]` in Cargo.toml
- [X] T004 [P] Create rustfmt configuration (edition 2021, max_width 100) in rustfmt.toml at repository root
- [X] T005 [P] Configure clippy lints (`warnings = "deny"`, opt-in to pedantic where appropriate) in Cargo.toml `[lints.clippy]` section
- [X] T006 [P] Add `/target` and `.cctmp/` (except `.gitkeep`) entries to .gitignore at repository root

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 全 user story が依存する共通基盤（Message 型、ULID、ルート解決、flock、CLI 骨格）。

**⚠️ CRITICAL**: T007〜T012 が揃うまで Phase 3 以降の実装着手は不可。

- [X] T007 [P] Define `Message { id, ts, from, body }` struct with serde derives and JSON serialization tests in src/message/record.rs
- [X] T008 [P] Implement `Message::new(from, body) -> Message` generating ULID + RFC3339 UTC timestamp in src/message/id.rs
- [X] T009 [P] Implement `resolve_root(cli_root: Option<&Path>, env: &HashMap<…>) -> PathBuf` honoring `--root` > `TSUJI_ROOT` > `$XDG_DATA_HOME/tsuji` > `~/.local/share/tsuji` in src/storage/paths.rs
- [X] T010 [P] Implement `with_exclusive_lock<F, R>(file: &File, f: F) -> Result<R>` wrapper around fs2 flock in src/storage/lock.rs
- [X] T011 Wire clap CLI skeleton with `send` / `read` / `channels` subcommand stubs (all returning `unimplemented!`) in src/main.rs
- [X] T012 Define shared CLI error type and exit-code mapping (`0` ok, `1` runtime, `2` arg syntax) in src/error.rs and re-export from src/lib.rs

**Checkpoint**: Foundation 完了 — US1〜US4 の並行着手が可能（テストファイルが独立しているため）。

---

## Phase 3: User Story 1 - セッション A → B のタスク受け渡し (Priority: P1) 🎯 MVP

**Goal**: 同一マシンの 2 セッション間で `tsuji send` ＋ `tsuji read` によりタスク文を受け渡せる最小フローを成立させる。

**Independent Test**: 異なる 2 シェルから `tsuji send` と `tsuji read` を叩き、送信したメッセージが受信側 stdout の JSON Lines に現れることを確認できる（spec User Story 1 / SC-001）。

### Tests for User Story 1 ⚠️ FIRST

> Write these tests FIRST and ensure they FAIL before implementation.

- [X] T013 [P] [US1] Author e2e test that runs `tsuji send --channel default --as agent-a --body "hello"` then `tsuji read --channel default` and asserts a JSON line containing `from:"agent-a"` and `body:"hello"` in tests/send_read_e2e.rs
- [X] T014 [P] [US1] Author contract test loading contracts/jsonl-schema.json (via `serde_json` + a JSON Schema validator like `jsonschema` dev-dep) and asserting every line from `tsuji read` matches the schema in tests/jsonl_schema_test.rs
- [X] T015 [P] [US1] Author test asserting two consecutive sends produce strictly increasing ULIDs (lexicographic) in tests/send_read_e2e.rs (separate `#[test]` fn)

### Implementation for User Story 1

- [X] T016 [US1] Implement `append_message(root, channel, msg)` using `OpenOptions::append + create` and `flock` (FR-007) in src/storage/writer.rs
- [X] T017 [US1] Implement `read_messages(root, channel) -> impl Iterator<Item=Result<String>>` streaming valid JSON lines and warning-skipping malformed lines (Edge Case "不正な JSON 行") in src/storage/reader.rs
- [X] T018 [US1] Implement `tsuji send` handler (validate channel name / sender / body, call append_message, exit 0 with empty stdout) in src/cli/send.rs
- [X] T019 [US1] Implement `tsuji read` handler (default JSON Lines pass-through to stdout) in src/cli/read.rs
- [X] T020 [US1] Wire send/read into clap dispatch in src/main.rs and rerun T013–T015 until all GREEN

**Checkpoint**: US1 単独で SC-001 を満たし、MVP として配布可能。

---

## Phase 4: User Story 2 - チャンネルを使い分ける (Priority: P2)

**Goal**: 複数チャンネル（例: `infra` / `research`）が物理的に分離した jsonl で動作し、`tsuji channels` で列挙可能。

**Independent Test**: 2 つの異なるチャンネル名で送信、それぞれを `tsuji read --channel <name>` で個別取得して混線していないことを確認。`tsuji channels` 出力に両方が含まれる。

### Tests for User Story 2 ⚠️ FIRST

- [X] T021 [P] [US2] Author e2e test asserting `tsuji send --channel newtopic ...` succeeds when `newtopic.jsonl` does not yet exist (auto-create) in tests/channels_e2e.rs
- [X] T022 [P] [US2] Author e2e test asserting `tsuji channels` lists existing channel names alphabetically (one per line) in tests/channels_e2e.rs
- [X] T023 [P] [US2] Author e2e test sending into channels `a` and `b` and asserting `tsuji read --channel a` returns only `a`'s messages in tests/channels_e2e.rs

### Implementation for User Story 2

- [X] T024 [US2] Extend `append_message` to `fs::create_dir_all(root)` and to create the channel file when missing (FR-006 / FR-016) in src/storage/writer.rs
- [X] T025 [US2] Implement `list_channels(root) -> Vec<String>` scanning `*.jsonl` under root and sorting in src/storage/reader.rs (or new src/storage/list.rs)
- [X] T026 [US2] Implement `tsuji channels` handler in src/cli/channels.rs and wire into clap dispatch in src/main.rs; rerun T021–T023 until GREEN

**Checkpoint**: US1 + US2 が共存。チャンネル混線なしで複数ストリームを運用できる。

---

## Phase 5: User Story 3 - 既読カーソルによる差分受信 (Priority: P2)

**Goal**: 受信側が `tsuji read --since <ULID>` を叩くと、それより後のメッセージだけが返り、コンテキストを汚さずに差分処理できる。

**Independent Test**: 10 件送信した後、5 件目の ID を渡して 6〜10 件目だけが返ることを確認。形式不正な `--since` は exit 1。

### Tests for User Story 3 ⚠️ FIRST

- [X] T027 [P] [US3] Author e2e test seeding 5 messages, capturing the 3rd ULID, then asserting `tsuji read --since <3rd>` returns exactly the 4th and 5th in tests/since_cursor.rs
- [X] T028 [P] [US3] Author e2e test passing a syntactically valid but non-existent ULID greater than all stored ones, asserting empty output + exit 0 in tests/since_cursor.rs
- [X] T029 [P] [US3] Author e2e test passing a malformed `--since` (e.g. `"not-a-ulid"`) and asserting exit code 1 with a stderr message in tests/since_cursor.rs

### Implementation for User Story 3

- [X] T030 [US3] Add `--since <ULID>` argument (with format validation regex `^[0-9A-HJKMNP-TV-Z]{26}$`) to `tsuji read` in src/cli/read.rs
- [X] T031 [US3] Implement `id > since` lexicographic filter inside `read_messages` (or a wrapping iterator) in src/storage/reader.rs and rerun T027–T029 until GREEN

**Checkpoint**: SC-004（カーソル意味論の完全性）が自動テストで担保された状態。

---

## Phase 6: User Story 4 - 人間による read-only 観察 (Priority: P3)

**Goal**: 人間ユーザが `tsuji read --follow --pretty` で会話を流し読みし、新着が 2 秒以内に表示される（SC-005）。

**Independent Test**: 観察用シェルで `tsuji read --channel default --follow --pretty` を開いた状態で別シェルから send し、2 秒以内に画面更新を目視確認（自動テストでも検証）。

### Tests for User Story 4 ⚠️ FIRST

- [X] T032 [P] [US4] Author e2e test asserting `--pretty` outputs lines matching `^\[<rfc3339>\] <from>: <first-body-line>$` (plus continuation lines for multi-line body) in tests/pretty_and_follow.rs
- [X] T033 [P] [US4] Author e2e test that spawns `tsuji read --follow` as a child process, sleeps 500ms, sends a message from another invocation, then asserts the child's stdout contains the new message within 2 seconds in tests/pretty_and_follow.rs
- [X] T033b [P] [US4] Author e2e tests that (a) spawn `tsuji read --follow --from-now` and assert pre-existing seed messages are NOT echoed, (b) new sends still surface within 2 seconds, and (c) `tsuji read --from-now` without `--follow` exits non-zero, all in tests/pretty_and_follow.rs

### Implementation for User Story 4

- [X] T034 [US4] Implement `pretty_format(msg: &Message) -> String` honoring multi-line bodies (continuation lines indented) in src/pretty.rs
- [X] T035 [US4] Add `--follow` and `--pretty` flags to `tsuji read` in src/cli/read.rs (clap definitions only)
- [X] T036 [US4] Implement `--follow` polling loop (poll interval ≤ 2 s; SIGINT-safe shutdown via `ctrlc` crate or signal-hook; if a new dep is needed add it in Cargo.toml first) in src/cli/read.rs and rerun T032–T033 until GREEN
- [X] T036b [US4] Implement `--from-now` flag (clap `requires = "follow"`; on entry establish cursor at the current tail and skip emitting existing messages) in src/cli/read.rs and rerun T033b until GREEN

**Checkpoint**: 全 4 ユーザストーリーが独立にテスト可能な状態。

---

## Phase 7: FR-014 — Claude plugin + Monitor tool bundle

**Goal**: 受信側 Claude のセッション開始時に Monitor tool が `tsuji read --follow --from-now` をバックグラウンド起動し、新着行を surface するための plugin を同梱する。

**Independent Test**: ビルド済み `tsuji` を PATH に通した状態で、Claude Code セッションが `claude-plugin/` を plugin 認識し、Monitor が起動して別シェルからの `tsuji send` を `/tasks` 上で観測できること。

- [X] T037 [P] Author Claude Code plugin manifest with name/version/description plus `userConfig.channel` and `experimental.monitors` pointing at `monitors/monitors.json` in claude-plugin/plugin.json
- [X] T038 [P] Author plugin-declared Monitor that runs `tsuji read --channel ${user_config.channel} --follow --from-now` with `when: "always"` in claude-plugin/monitors/monitors.json
- [X] T038b Remove the legacy `/loop`-based skill (`trash claude-plugin/skills/tsuji-listen.md` and the now-empty `claude-plugin/skills/`)
- [ ] T039 Smoke-test the Monitor by installing `claude-plugin/` into a local Claude Code session, setting `user_config.channel` to `smoke`, sending a message from another shell, and confirming the Monitor surfaces it (and that pre-existing messages are NOT replayed); record observations in .cctmp/scratch/monitor-smoke.md

---

## Phase 8: SC-003 — Concurrent send hardening

**Goal**: 並行 `send` 100 回連続でログ破損ゼロを自動テストで担保する。

- [X] T040 Author concurrency test spawning 2 OS threads, each invoking `tsuji send` 50 times against the same channel under a `tempfile::tempdir()` root, then asserting (a) line count = 100, (b) every line parses, (c) ULIDs are unique in tests/concurrent_send.rs
- [X] T041 If T040 fails, harden `append_message` (e.g. acquire `lock_exclusive` before write, ensure `write_all` happens as single syscall via pre-formatted buffer, fsync if necessary) in src/storage/writer.rs and rerun T040 until GREEN

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: ドキュメント、品質ゲート、quickstart 手動検証。

- [X] T042 [P] Write README.md with one-paragraph overview, install command (`cargo install --path .`), and a link to specs/001-tsuji-chat-cli/quickstart.md at repository root
- [X] T043 [P] Add rustdoc comments to all `pub` items in src/lib.rs, src/message/**.rs, src/storage/**.rs
- [X] T044 Run `cargo fmt --check` from repository root and fix any formatting drift
- [X] T045 Run `cargo clippy --all-targets -- -D warnings` from repository root and fix all warnings
- [X] T046 Run `cargo test` from repository root and verify the entire suite passes (unit + integration + concurrency)
- [X] T047 Run `cargo build --release` from repository root and confirm the artifact at target/release/tsuji starts and responds to `--help`
- [X] T048 Walk through specs/001-tsuji-chat-cli/quickstart.md sections 2–6 manually against the release binary and record outcomes in .cctmp/scratch/quickstart-walkthrough.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No deps — start first.
- **Phase 2 (Foundational)**: Depends on Phase 1. BLOCKS Phase 3+.
- **Phase 3 (US1, P1)**: Depends on Phase 2. MVP scope.
- **Phase 4 (US2, P2)**: Depends on Phase 2. Reuses writer/reader from Phase 3 (T024 extends T016, T025 reuses reader infra) — best run after Phase 3.
- **Phase 5 (US3, P2)**: Depends on Phase 2; lightly extends reader from Phase 3 (T031 extends T017) — best after Phase 3.
- **Phase 6 (US4, P3)**: Depends on Phase 2; lightly extends reader from Phase 3 — best after Phase 3.
- **Phase 7 (FR-014 plugin + Monitor)**: Depends on Phase 6 (Monitor invokes `tsuji read --follow --from-now`, which exists after T036b).
- **Phase 8 (SC-003 concurrency)**: Depends on Phase 3 (writer implemented).
- **Phase 9 (Polish)**: Depends on all desired phases complete.

### User Story Dependencies

- US1 (Phase 3): Depends only on Foundational (Phase 2). No upstream user stories.
- US2 (Phase 4): Independently testable. Shares writer with US1 but the channel-isolation test does not depend on US1 acceptance.
- US3 (Phase 5): Independently testable. Shares reader with US1 but operates on a `--since` flag added in Phase 5.
- US4 (Phase 6): Independently testable. Adds `--follow`/`--pretty` without altering default behavior used by US1/US2/US3.

### Within Each User Story

- Write all tests in the story's "Tests for User Story X" subsection and verify they FAIL.
- Implement in the listed order (models → storage → CLI → wiring).
- Rerun the story's tests until GREEN before moving on.

### Parallel Opportunities

- All Phase 1 tasks marked [P] (T003–T006) can run in parallel after T001/T002.
- All Phase 2 tasks marked [P] (T007–T010) touch different files and can run in parallel.
- Tests within each story phase marked [P] are in either the same test file (different `#[test]` fns, can be authored independently) or different files; safe to parallelize.
- Once Foundational completes, US1–US4 phases CAN proceed in parallel if multiple worktrees are used; default ordering is sequential P1 → P2 → P2 → P3.
- Phase 7 (plugin authoring) and Phase 8 (concurrency hardening) can run in parallel once Phase 3 is green.

---

## Parallel Example: User Story 1

```bash
# After Phase 2 is green, draft all three failing tests in parallel:
Task: "Author e2e send/read test in tests/send_read_e2e.rs"      # T013
Task: "Author contract schema test in tests/jsonl_schema_test.rs" # T014
Task: "Author ULID monotonicity test in tests/send_read_e2e.rs"   # T015

# Then implement core modules in parallel (Phase 2 already provides Message + paths + lock):
Task: "Implement writer (flock + append) in src/storage/writer.rs" # T016
Task: "Implement reader (line stream) in src/storage/reader.rs"     # T017

# Sequentially wire CLI handlers and rerun tests:
Task: "Implement send handler in src/cli/send.rs"  # T018
Task: "Implement read handler in src/cli/read.rs"  # T019
Task: "Wire into main dispatch in src/main.rs"     # T020
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 (Setup).
2. Complete Phase 2 (Foundational).
3. Complete Phase 3 (US1).
4. **STOP and VALIDATE**: 2 つのシェルで send/read を叩いて手動確認。
5. これだけで SC-001 を満たした MVP として `tsuji` を使い始められる。

### Incremental Delivery

1. Setup ＋ Foundational → 基盤完成。
2. US1 → MVP リリース。
3. US2（チャンネル）→ 用途別運用が可能に。
4. US3（カーソル）→ コンテキスト効率が改善し、Claude セッションでの実運用に耐える。
5. US4（観察）＋ Phase 7（plugin + Monitor tool）→ 「人間が見守りつつ Claude 同士が会話」の本来の体験が完成。
6. Phase 8（並行ストレス）＋ Phase 9（Polish）→ 1.0 リリース。

### TDD ループ (t_wada 流)

各 story phase の「Tests …」セクションを先に書き、`cargo test --test <name>` で FAIL を確認 → Implementation セクションを順に進めて GREEN → 必要なら refactor → 次の story へ。

---

## Notes

- 全タスクが ≤ 50。MVP（Phase 1–3）の見積もりは ~20 タスク。
- [P] タスクは別ファイル／別関数を触るので、worktree 並列化や複数 Claude セッションで分担可能。
- 各タスク完了後、または論理単位ごとに git commit（Conventional Commits ＋ プロンプト原文を含む形式）。
- 検証用スクリプトや手動確認の記録は `.cctmp/scratch/` に置く（global rule）。
- `rm` 禁止：クリーンアップは `trash` を使う（global rule）。
- `cargo test`、`cargo clippy -D warnings`、`cargo fmt --check`、`cargo build --release` の 4 つが全部通る状態で完了とみなす。
