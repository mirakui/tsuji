# Phase 1 Data Model: tsuji

**Feature**: 001-tsuji-chat-cli

**Date**: 2026-05-22

spec.md の Key Entities を実装視点で具体化したもの。Phase 2 のタスク分解および契約テスト（`contracts/jsonl-schema.json`）の参照元となる。

## Entities

### Message

不可変イベント。1 行 1 メッセージで JSON Lines ファイルへ append される。

| Field | Type | Required | Constraints / Notes |
|---|---|---|---|
| `id` | string | yes | ULID（26 文字、Crockford Base32）。チャンネル内で一意。辞書順＝時系列順を満たす。 |
| `ts` | string | yes | RFC3339 / ISO 8601。UTC 推奨（`Z` サフィックス）。生成は `chrono::Utc::now().to_rfc3339()`。 |
| `from` | string | yes | 送信者の自称名。任意の非空文字列。バリデーションは「1 文字以上、改行を含まない、最大 64 文字」を初期ルールとする（実装時に確認）。 |
| `body` | string | yes | 改行を含む任意長プレーンテキスト。JSON 文字列としてエスケープ保存。0 長は禁止（バリデーションエラー）。最大長は v1 では明示制限を設けないが、1 メッセージあたり 1 MiB を soft cap として警告対象に。 |

**Serialization**: `serde_json` で 1 行に直列化。改行（`\n`）は JSON 文字列内ではエスケープされるため、行の境界とは衝突しない。

**Lifecycle**: 作成のみ。編集・削除は v1 サポートなし（append-only）。

### Channel

論理的なチャット空間。物理実体はチャンネル名と同名の JSON Lines ファイル `<root>/<channel>.jsonl`。

| Field | Type | Required | Constraints / Notes |
|---|---|---|---|
| `name` | string | yes | チャンネル名。許容文字集合は `[a-zA-Z0-9_-]+`、長さ 1〜64。それ以外は CLI バリデーションでエラー。 |
| `path` | path | derived | `<root>/<name>.jsonl`。`root` は FR-015 のルール（`--root` > `TSUJI_ROOT` > XDG）で解決。 |

**Existence rule**: チャンネルの存在 = 当該 `.jsonl` ファイルの存在。`send` 時にファイルが無ければ親ディレクトリごと作成（FR-016）。`read` で存在しないチャンネル名を指定された場合は「メッセージ 0 件」を意味する空出力＋ exit 0（spec Edge Case「存在しないチャンネル読み込み」）。

**Listing rule**: `tsuji channels` は `<root>` 直下を走査し、拡張子 `.jsonl` のファイル名から拡張子を除いた一覧を辞書順で出力（FR-008）。

### Sender (Participant)

`Message.from` の値で識別される論理的な発話主体。物理セッション ID とは紐付けず、識別はラベル名のみ。重複は禁止しない（運用判断）。

### Cursor

受信側が「ここまで読んだ」状態を表す ULID 文字列。**システムは保持しない**（ステートレス）。クライアント（呼び出し側 Claude / skill）が `.tsuji-cursor` 等のファイルやセッション内変数で保持し、`--since <cursor>` に渡す。

**Semantics**: `read --since X` は厳密に「`id > X` を満たすメッセージのみ返す」（ULID の辞書順比較）。`X` と完全一致するメッセージは含まれない。

## Relationships

- Channel **1** ── **N** Message（Channel 1 本に複数の Message が時系列で append される）
- Channel **N** ── **M** Sender（同一 Sender が複数 Channel に書ける／同一 Channel に複数 Sender が書ける）

## State Transitions

v1 では Message が「append される → 不可変に存在する」のみ。Channel も「存在しない → 最初の `send` で作成 → 以降存在し続ける」のみ。明示的なアーカイブ／削除は v1 範囲外。

## Validation Rules（実装時に CLI レイヤで実施）

1. `channel` 引数: `^[a-zA-Z0-9_-]{1,64}$` にマッチしない場合エラー。
2. `as` 引数: 非空・改行なし・長さ ≤ 64。違反時エラー。
3. `body` 引数（または stdin）: 非空。完全に空の場合エラー（誤送防止）。
4. `--since` 引数: 26 文字の Crockford Base32 ULID 形式。形式不正は exit code 非ゼロ。値が存在しないメッセージを指していた場合は「以降のメッセージを返す」を素直に実行し、エラーにはしない（Edge Case ハンドリングと整合させる）。
5. `--root` 引数: 存在しないパスは `send` 時に作成、`read` 時にはエラー扱いとせず空出力。

## Implications for Source Layout

- `src/message/record.rs`: `Message` 構造体 + `serde` derive + コンストラクタ。
- `src/message/id.rs`: ULID 生成と検証。
- `src/storage/paths.rs`: `Channel.path` 解決、root 解決。
- `src/storage/writer.rs`: flock + append + JSON 直列化。
- `src/storage/reader.rs`: 行ストリーム + `--since` フィルタ。
- `src/cli/*`: 上記のバリデーションを各サブコマンドの入り口で行う。
