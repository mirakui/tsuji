# CLI Contract: tsuji

**Feature**: 001-tsuji-chat-cli

**Date**: 2026-05-22

`tsuji` バイナリが外部（Claude Code Bash ツール／人間ユーザ／同梱の Claude skill）に対して提供する CLI インタフェースの契約。Phase 2 の契約テスト（`tests/`）はこの文書を基準に作成する。

## Global Options

すべてのサブコマンドに共通：

| Option | Type | Default | Description |
|---|---|---|---|
| `--root <PATH>` | path | env or XDG | チャンネルログのルートディレクトリを上書き。最優先。 |
| `--help`, `-h` | flag | — | ヘルプ表示。サブコマンドごとにも有効。 |
| `--version`, `-V` | flag | — | `tsuji <semver>` を 1 行で出力して exit 0。 |

環境変数：

| Variable | Description |
|---|---|
| `TSUJI_ROOT` | `--root` 未指定時のルート。`XDG_DATA_HOME` よりも優先。 |
| `XDG_DATA_HOME` | 最終フォールバックの親（標準）。未設定時は `~/.local/share`。 |

## Exit codes

| Code | 意味 |
|---|---|
| 0 | 成功（メッセージ 0 件の `read` を含む） |
| 1 | 一般エラー（バリデーション失敗・I/O 例外） |
| 2 | CLI 引数の構文エラー（clap が返す既定値） |

stderr には 1 行以上の人間可読なエラーメッセージを出す（FR-012）。

## Subcommands

### `tsuji send`

メッセージを 1 件送信する。

```text
tsuji send --channel <NAME> --as <SENDER> [--body <TEXT> | -]
```

| Arg | Required | Description |
|---|---|---|
| `--channel <NAME>` | yes | 送信先チャンネル名。`^[a-zA-Z0-9_-]{1,64}$`。 |
| `--as <SENDER>` | yes | 自称名。非空・改行なし・長さ ≤ 64。 |
| `--body <TEXT>` | yes (1) | 本文。改行を含む任意長テキスト。 |
| `-` (stdin) | yes (1) | `--body` の代替として stdin から読み取る。 |

(1) `--body` と stdin はどちらか一方必須。両方／両方欠如はエラー（exit 2 または 1）。

**Behavior**:

1. ルート解決（`--root` > `TSUJI_ROOT` > `XDG_DATA_HOME/tsuji` > `~/.local/share/tsuji`）。
2. `<root>/<channel>.jsonl` を `O_CREATE | O_APPEND` で開く。親ディレクトリが無ければ `mkdir -p` 相当を実行（FR-016）。
3. ULID と `ts`（RFC3339 UTC）を生成。
4. ファイルに `flock(LOCK_EX)` をかける。
5. `{"id":...,"ts":...,"from":...,"body":...}\n` を 1 回の `write_all` で書く（FR-007）。
6. `flush()` → `unlock` → close。
7. **stdout には何も出さない**（成功時は exit 0 のみ）。エラー時は stderr。

**Errors**:

- バリデーション違反: exit 1、stderr に「`channel: invalid name 'XX!!' (allowed: [a-zA-Z0-9_-]{1,64})`」のような明示メッセージ。
- I/O 失敗: exit 1、エラーチェーンを stderr に。

### `tsuji read`

メッセージを取得する。デフォルトはチャンネル先頭から末尾まで、JSON Lines で stdout に出力。

```text
tsuji read --channel <NAME> [--since <ULID>] [--pretty] [--follow]
```

| Arg | Required | Description |
|---|---|---|
| `--channel <NAME>` | yes | 取得対象。 |
| `--since <ULID>` | no | 指定 ULID `より後` のメッセージのみ返す（境界除外）。形式不正は exit 1。指す ULID が存在しないメッセージでもエラーにはならず、辞書順 > 指定値の行を返す（spec Edge Case）。 |
| `--pretty` | no | 出力形式を人間可読に切り替える（FR-018）。各メッセージを `[<ts>] <from>: <body>` 形式の複数行ブロックで出力。 |
| `--follow` | no | ファイル末尾到達後も最大 2 秒以下のポーリング間隔で監視を続け、新着メッセージを逐次出力（FR-010、SC-005）。`SIGINT` で停止。 |

**Behavior**:

1. ルート解決。
2. `<root>/<channel>.jsonl` を読み取り専用で開く。
   - 存在しない場合: stdout 空、exit 0。
3. 行をストリーム読みし、`--since` 指定があれば `id > <since>`（文字列辞書順）を満たす行のみ採用。
4. `--pretty` 未指定時はパースした JSON Lines を **そのまま** stdout に書き出す（serde で再シリアライズし、未知フィールドの将来互換を保つ）。
5. `--pretty` 指定時は人間可読フォーマットで書き出す。
6. `--follow` 指定時は EOF 後も sleep + tail を継続。

**Errors**:

- 不正な `--since` 形式: exit 1。
- 既存ファイルだが不正行を含む: 不正行をスキップしつつ正常行を出力し、stderr に warning（Edge Case「不正な JSON 行」）。exit code は 0（処理続行）。
- I/O 失敗: exit 1。

### `tsuji channels`

既存チャンネル名の一覧を出力する。

```text
tsuji channels
```

**Behavior**:

1. ルート解決。
2. `<root>` 直下の `*.jsonl` を列挙、拡張子を除去、辞書順ソート、改行区切りで stdout。
3. ルートが存在しない／空の場合: stdout 空、exit 0。

**Errors**: I/O 失敗時 exit 1。

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

## Output schema (machine-readable)

`tsuji read`（`--pretty` 未指定）の 1 行は `contracts/jsonl-schema.json` に定義する JSON オブジェクトと一致する。

## 非サブコマンド

`tsuji` 単独実行（引数なし）はヘルプ表示 ＋ exit 0、または exit 2（clap 既定）。一貫性のため exit 0 ＋ ヘルプ表示にする。

## 互換性ポリシー

- v1 では `id` / `ts` / `from` / `body` の 4 フィールドのみを公式契約とする。
- 将来追加されるフィールドはデフォルト値で読み手が無視できることを保証（forward-compat）。
- フィールド名の変更・削除は破壊的変更とし、メジャーバージョンの引き上げを伴う。
