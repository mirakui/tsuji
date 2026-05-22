# Phase 0 Research: tsuji

**Feature**: 001-tsuji-chat-cli

**Date**: 2026-05-22

Phase 0 では plan.md の Technical Context を確定させるための調査を行う。各項目について Decision / Rationale / Alternatives considered の三点で記述する。NEEDS CLARIFICATION は残っていない（すべて事前 clarification で解消済み）。

## 1. ULID 実装（Rust）

- **Decision**: [`ulid`](https://crates.io/crates/ulid) クレート（`Ulid::new()` で時刻＋ランダム生成、`Display` で 26 文字の Crockford Base32 表現）を採用する。
- **Rationale**:
  - 標準的な 128-bit ULID 仕様に準拠。
  - 文字列表現は辞書順＝時系列順を満たすので、`--since <id>` のカーソル比較を単純な文字列比較で実装できる（spec の Clarification Q1 と整合）。
  - `serde` feature を有効にすれば `Message` 構造体の serialize/deserialize が自動で扱える。
  - 同一ミリ秒内の衝突は単調モノトニック乱数で実質ゼロ（ローカル単機構成）。
- **Alternatives considered**:
  - `uuid` クレートの v7（時刻順序ソート可）→ 表現が長く、辞書順比較の仕様は ULID よりやや非自明。Clarification で ULID 採用が決まっているため不要。
  - 自前のタイムスタンプ＋乱数生成 → 仕様の自己定義コストとテスト負担に見合わない。

## 2. 並行 `send` の atomicity（flock + O_APPEND）

- **Decision**: チャンネルファイルを `OpenOptions::new().append(true).create(true).open(path)` で開き、書き込み前に `fs2::FileExt::lock_exclusive` で flock を取得し、1 メッセージ = 1 行を `write_all` してから flush・unlock する。
- **Rationale**:
  - POSIX における `O_APPEND` は単一の `write(2)` が atomic に末尾追記される保証を持つが、これは「カーネル内のシーク+書き込みが atomic」というだけで、複数バイトをユーザ空間で `serde_json` 等で組み立てる過程ではプロセス間で順序が乱れうる。
  - また、書き込みサイズが PIPE_BUF（POSIX で 512 〜 macOS の 8192）を超える場合、`O_APPEND` 単独で 1 行の atomic 性は保証されない。本仕様は改行を含む任意長テキスト（FR-017）を受理するため、PIPE_BUF を超えるメッセージが現実的に発生する。
  - `flock(2)` の advisory lock は同一ホスト・同一ファイルへの並行プロセスからの送信を相互排他化でき、`fs2` クレートで OS 抽象化される。
  - macOS と Linux はいずれも flock を実装するが、ネットワーク FS では未保証なので Target Platform を local FS に限定する。
- **Alternatives considered**:
  - O_APPEND 単独（ロックなし）→ 仕様 SC-003（100 回連続並行で破損 0）を満たせない可能性。
  - `rename(2)` ベースの atomic write（temp + rename）→ append-only ログとの相性が悪く、毎回全コピーになる。
  - SQLite / 専用 DB → 「サーバ・依存ゼロ・JSON Lines」というコンセプトに反する（FR-001/FR-009）。

## 3. JSON Lines のレコード形式

- **Decision**: 1 行 1 メッセージ。フィールドは固定キー `{"id":"<ULID>","ts":"<RFC3339>","from":"<sender>","body":"<text>"}`。本文は JSON 文字列としてエスケープ済み（改行は `\n`）。
- **Rationale**:
  - パーサが容易（`serde_json::Deserializer::from_str` 単一行）。
  - フィールド名を短く保ち、人間が `--pretty` 無しでも生ファイルを `tail -f` で読めるバランスを取った。
  - 将来の拡張に備え、未知フィールドは `serde(default)` で無視する設計とし、forward-compat を確保。
- **Alternatives considered**:
  - 圧縮バイナリ形式（msgpack 等）→ 人間が `tail -f` で観察可能（FR-010）にできない。
  - `body` を構造化（オブジェクト）→ Clarification Q4 で v1 はテキストのみと決定済み。

## 4. ルートディレクトリ解決

- **Decision**: 解決順は (1) `--root <path>` CLI フラグ、(2) `TSUJI_ROOT` 環境変数、(3) `XDG_DATA_HOME/tsuji/` または `~/.local/share/tsuji/`。最初に見つかった非空の値を採用。存在しないディレクトリは `send` 時に `fs::create_dir_all` で自動作成（FR-016）。
- **Rationale**:
  - XDG Base Directory Specification はユーザ global なツールデータ置き場として広く使われ、衝突しない。
  - 環境変数 ＋ CLI フラグの両方を許すと、（a）shell rc に `TSUJI_ROOT=...` で個人のデフォルトを切り替え、（b）一回限りの `--root` で挙動を上書き、という二段階の柔軟性が得られる。
  - `dirs` クレートまたは自前の `std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home))` で簡潔に実装可能。
- **Alternatives considered**:
  - プロジェクト直下 `.tsuji/` を自動探索 → Clarification Q3 で global を選択済み。
  - 完全に環境変数のみ → CLI フラグでの一時オーバーライドができず CI/テストで不便。

## 5. Claude Code plugin / skill marketplace 配布

- **Decision**: リポジトリ内 `claude-plugin/` に Claude Code plugin として `plugin.json`（plugin manifest）と `skills/tsuji-listen.md` を配置し、Anthropic 公式の skill marketplace（あるいは v1 時点で利用可能な等価な配布チャネル）に登録する。skill は `tsuji read --since <last_id>` をワンショット実行し、`/loop` （ダイナミックループ／`ScheduleWakeup`）で自分自身を再スケジュールする。
- **Rationale**:
  - Clarification Q2 で `/loop` ベースの方針が確定（Stop hook 不採用）。
  - skill を marketplace 経由で配ると、受信側 Claude のセッションで `/plugin install tsuji` 一発で導入できる（実コマンド名は実装時に確認）。
  - skill 本体は markdown + 短いプロンプトで済み、CLI とは独立にメンテナンスできる（CLI が後方互換を維持する限り、skill を更新せずに済む）。
- **Alternatives considered**:
  - skill を `~/.claude/skills/` に手動コピー → marketplace 経由の方が他環境への展開・再現性が高い。
  - MCP server として実装 → 受信に常駐プロセスを要するため FR-009 の「常駐ゼロ」を破る。`/loop` の方が思想と合致する。
  - Stop hook 連携 → Q2 で明確に不採用。

### 5.1 `/loop` を使った skill の構造（暫定スケッチ）

`tsuji-listen.md` の中身（実装時に確定する暫定案）:

```markdown
---
name: tsuji-listen
description: Polls a tsuji channel and surfaces new messages to the current Claude session.
arguments:
  - name: channel
    description: Channel name to listen on
    required: true
---

You are running the tsuji listener for channel "{channel}".

1. Read the last-seen ULID from .tsuji-cursor (or use "" if absent).
2. Run: `tsuji read --channel {channel} --since <cursor>` and parse the JSON Lines output.
3. If new messages exist, surface their bodies to the user/agent context and update .tsuji-cursor to the latest id.
4. Re-schedule yourself via /loop (ScheduleWakeup) with a 60–300 second delay.
```

カーソル永続化先、ポーリング間隔の動的調整、停止条件などの詳細は Phase 2（tasks）以降の判断に委ねる。

## 6. CLI フレームワーク選定（clap）

- **Decision**: `clap` 4.x の `derive` API を採用。
- **Rationale**:
  - Rust エコシステムでデファクト。`tsuji send --channel <name> --as <sender> [body]` のような複合サブコマンドを宣言的に書ける。
  - `--help` 自動生成で SC-006（README なしで使い方に 1 分以内到達）を支援。
  - 安定版・MSRV も合理的。
- **Alternatives considered**:
  - `argh` / `pico-args` → 軽量だが derive と自動ヘルプの完成度で劣る。
  - 手書き argparse → メンテナンスコスト過大。

## 7. テスト戦略

- **Decision**:
  - ユニット: 各モジュール内で純粋関数（パス解決、ULID 生成のラップ、JSON 行のパース・整形）をカバー。
  - インテグレーション: `assert_cmd` + `predicates` + `tempfile` で `tsuji` バイナリを子プロセスとして叩き、stdout/stderr/exit code を検証。
  - 並行性: `concurrent_send.rs` で 2 プロセス × 50 回ずつ並行 `send` を実行し、ログ行数 100、JSON パース成功率 100% を assert（SC-003）。
  - TDD（t_wada 流）に従い、各 user story の受入シナリオを先にテスト化してから実装着手。
- **Rationale**:
  - `assert_cmd` は出力差分・終了コードのアサートが宣言的で読みやすく、TDD 駆動に向く。
  - 並行テストは flock の効力検証として不可欠（SC-003 を 1 件以上の自動テストで担保）。
- **Alternatives considered**:
  - 手書き shell スクリプトでの e2e → 移植性低、デバッグ困難。
  - `cargo nextest` → 並列実行を tweak したくなる時点で導入検討、初期は不要。

## 8. lint / formatter / build

- **Decision**: `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo build --release`、`cargo test`。これら 4 つを「完了条件」とする。
- **Rationale**: global rule「テスト、lint、formatter、build が通る状態で引き渡す」と整合。
- **Alternatives considered**: 追加ツール（`cargo-audit` 等）は意義はあるが個人用途では過剰。後段で検討。

---

以上で Phase 0 の調査は完了。NEEDS CLARIFICATION 残存なし、Constitution Check 違反なし。次は Phase 1（data-model / contracts / quickstart 生成）へ進む。
