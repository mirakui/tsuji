# Implementation Plan: tsuji — Inter-Session Chat CLI for Claude Code

**Branch**: `001-tsuji-chat-cli` | **Date**: 2026-05-22 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-tsuji-chat-cli/spec.md`

## Summary

ローカルで起動中の複数の Claude Code セッション間で「これやっといて」とタスクを受け渡せる、サーバ不要のファイルベース chat CLI を Rust で実装する。チャンネルログは ULID 付きの JSON Lines として `$XDG_DATA_HOME/tsuji/` 配下に保存し、`flock(2)` ベースの排他制御で並行 `send` の atomicity を担保する。受信側は ワンショットの `tsuji read --since <ULID>` で差分取得でき、継続ポーリングは同梱の Claude Code plugin が Monitor tool 経由で `tsuji read --follow --from-now` をバックグラウンド実行することで自動化する（FR-014）。

## Technical Context

**Language/Version**: Rust（stable、1.79 以降を最小要件と仮定）

**Primary Dependencies**:

- `clap` (subcommand パース。`derive` feature)
- `ulid` (ULID 生成、`Display` でそのまま 26 文字文字列化)
- `serde` + `serde_json` (JSON Lines シリアライズ／パース)
- `chrono` (タイムスタンプ; RFC3339 文字列で記録)
- `fs2` (`FileExt::try_lock_exclusive` / `lock_exclusive` で `flock(2)` 抽象化)
- `anyhow` (エラーハンドリング)
- 標準 `std::fs`（O_APPEND での書き込み）

**Storage**: ローカルファイルシステム。1 チャンネル = 1 JSON Lines ファイル。デフォルトルートは `$XDG_DATA_HOME/tsuji/`（未設定時 `~/.local/share/tsuji/`）。`--root` / `TSUJI_ROOT` で上書き可（FR-015）。

**Testing**:

- `cargo test` ベース。
- ユニットテスト: 各モジュール (`#[cfg(test)] mod tests`)。
- インテグレーションテスト: `tests/` 直下に CLI を `assert_cmd` で起動して JSON Lines 出力を検証する e2e テスト。
- 並行性テスト: `tempfile` でテンポラリディレクトリを作り、複数スレッド／プロセスから `send` を叩いてログ破損が無いことを確認。
- カバレッジ目標 80%+（global rules に従う）。

**Target Platform**: macOS (Darwin)、Linux。POSIX `flock(2)` 前提のためネットワーク FS / Windows は v1 対象外（CLAUDE.md global rules には記載なし、本プロジェクトの仮定）。

**Project Type**: 単一の Rust CLI クレート + 同梱の Claude Code plugin（marketplace 配布）。

**Performance Goals**:

- SC-002: `tsuji send` / `tsuji read` のコールドスタート〜出力までを、数百〜数千メッセージのチャンネルで 1 秒未満。
- SC-005: `--follow` 時の新着検知遅延 2 秒以内（ポーリング間隔の上限）。

**Constraints**:

- サーバ・常駐デーモン非依存（FR-009）。
- 並行 `send` 100 回連続でログ破損ゼロ（SC-003）— `flock` の排他取得 + `O_APPEND` で達成。
- 認証・暗号化なし（FR-011、ローカル前提）。
- 改行を含むメッセージ本文を JSON 文字列エスケープで保持（FR-017）。

**Scale/Scope**: 個人用途、同時稼働セッション数 ≤ 10、チャンネル数 ≤ 20、チャンネルあたりメッセージ数は数千〜1 万程度を上限の現実シナリオと想定。これを超えるスケールはローテーション機構（v1 範囲外）の検討対象。

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

`.specify/memory/constitution.md` は未記入のテンプレート状態であり、当プロジェクト固有の憲法上の拘束はまだ存在しない。したがって明示的な gate は無く、global rules（CLAUDE.md、git-workflow.md、package-managers.md）に従う：

- **TDD（t_wada 流）**: 全機能を先にテスト → fail を確認 → 実装の順で進める。
- **Conventional Commits** ＋ プロンプト原文 (Japanese) を commit メッセージに含める。
- **`rm` 禁止**: `trash` を使う（オペレーション時）。
- **テスト / lint / formatter / build をすべて通す**: `cargo test`、`cargo clippy -- -D warnings`、`cargo fmt --check`、`cargo build`。

violation なし。

## Project Structure

### Documentation (this feature)

```text
specs/001-tsuji-chat-cli/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/           # Phase 1 output (/speckit-plan command)
│   ├── cli.md           # tsuji CLI のサブコマンド・引数・終了コード仕様
│   └── jsonl-schema.json  # チャンネルログ 1 行の JSON Schema
├── checklists/
│   └── requirements.md  # spec quality checklist
└── tasks.md             # Phase 2 output (/speckit-tasks command - NOT created here)
```

### Source Code (repository root)

```text
Cargo.toml
src/
├── main.rs              # CLI エントリ。clap::Parser でサブコマンド分岐
├── cli/
│   ├── mod.rs
│   ├── send.rs          # `tsuji send` 実装（FR-002, FR-007, FR-017）
│   ├── read.rs          # `tsuji read` 実装（FR-003, FR-004, FR-018）
│   ├── follow.rs        # `tsuji read --follow` 実装（FR-010）
│   └── channels.rs      # `tsuji channels` 実装（FR-008）
├── storage/
│   ├── mod.rs
│   ├── paths.rs         # ルート解決（FR-015 / --root / TSUJI_ROOT / XDG）
│   ├── lock.rs          # flock 抽象（fs2 ラッパ）
│   ├── writer.rs        # atomic append（FR-007）
│   └── reader.rs        # JSON Lines 読み出し＋--since 差分（FR-004）
├── message/
│   ├── mod.rs
│   ├── id.rs            # ULID 生成（FR-005）
│   └── record.rs        # Message struct + serde 定義
└── pretty.rs            # `--pretty` 人間可読フォーマット（FR-018）

tests/
├── send_read_e2e.rs     # User Story 1 受入テスト
├── channels_e2e.rs      # User Story 2 受入テスト
├── since_cursor.rs      # User Story 3 / FR-004 / Edge Case
├── concurrent_send.rs   # SC-003 並行送信
└── pretty_and_follow.rs # FR-010 / FR-018

claude-plugin/           # Claude Code marketplace 配布アーティファクト
├── plugin.json          # plugin manifest
└── monitors/
    └── monitors.json    # plugin-declared Monitor: `tsuji read --follow --from-now`

.cctmp/scratch/          # 任意検証スクリプト置き場（global rule）
```

**Structure Decision**: 単一 Rust crate（`tsuji`）＋ 同リポジトリ内 `claude-plugin/` ディレクトリで Claude Code plugin を同梱する 1 プロジェクト構成を採用。CLI / storage / message の 3 モジュールに分け、テストは `tests/` 直下に e2e を、各モジュール内に unit テストを置く。plugin は `plugin.json` + `monitors/monitors.json` の 2 ファイル構成で、Monitor が CLI を直接呼ぶため CLI 互換性さえ保てば plugin 側のロジックは不要。

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

該当なし。憲法は未記入のためゲート違反なし。
