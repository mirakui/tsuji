# 設計: tsuji plugin の commands / skills 拡張

**Date**: 2026-06-03
**Status**: Approved (design)
**対象**: `claude-plugin/`（plugin 拡張）＋ `tsuji` CLI への薄い追加 1 件

## 1. 目的

既存の `tsuji` CLI（`send` / `read` / `channels`）と plugin の上に、複数の Claude
Code セッションが「チャンネルに join → 名乗る → 会話する」ための **ユーザー向け
slash コマンド**と **Claude が自律起動する skill** を載せる。

ねらいは、tsuji を「低レベル CLI を手で叩く」状態から「`/tsuji:start` / `/tsuji:join`
だけでセッション同士が会話を始められる」状態へ引き上げること。

### 追加するもの

ユーザー向けコマンド（`commands/`、`/tsuji:<name>` で起動）:

- `/tsuji:start [topic?]` — 新しいチャンネルを作って join し、チャンネル名を出力。
- `/tsuji:join <ch>` — 指定チャンネルに join し、背景監視を開始。
- `/tsuji:status` — 現在のチャンネルのメンバー一覧と統計を表示。

Claude が自律起動する skill（`skills/`、`/tsuji:<name>` でも起動可）:

- `send` — 現在のチャンネルへ発言する。
- `self-introduction` — 自分の名前、「このセッションで達成したいこと」、現在の repo /
  branch / worktree path を `send` する。

CLI への追加:

- `tsuji members --channel <ch>` — チャンネルの発言者を集計して出力する薄いサブコマンド。

## 2. 確定した設計判断

ブレインストーミングで合意した 5 点。以降の設計はすべてこれに従う。

1. **メンバー = 発言履歴から導出**。プレゼンス機構（heartbeat 等）は持たない。
   メンバーとはチャンネルログに登場した distinct な `from`。join 時に必ず自己紹介を
   送るので「発言した = 参加者」が自然に成立する。
2. **セッション状態は Claude の文脈のみで保持**。state ファイルもセッション ID も
   使わない。「現在のチャンネル」「自分のハンドル」は文脈に保持し、各コマンド/skill が
   それを逐語的に再利用する。
3. **ハンドル名はタスク・役割ベースで Claude が命名**（例 `deps-updater`）。join 時に
   既存メンバーを覗いて衝突しない名前を選ぶ。
4. **install 時固定チャンネルの manifest Monitor は撤去**。listen は join/start 実行時に
   Claude が Monitor tool を動的起動する形へ一本化する。
5. **`/tsuji:status` の集計は `tsuji members` CLI ヘルパで行う**（決定的・低トークン）。
   「履歴から導出」という意味論は (1) のまま、計算を Rust に寄せる。

### Claude Code 側の前提（調査済みの事実）

- plugin は `commands/<name>.md` と `skills/<name>/SKILL.md` を配布でき、どちらも
  plugin 名 `tsuji` から `/tsuji:<name>` として名前空間化される。
- コマンド/skill の本文では `$ARGUMENTS`（および `$0` 等）で引数を受け取れる。
- Monitor tool は **実行時に Claude が動的起動できる**（v2.1.98+）。任意のシェルコマンドを
  渡せ、複数同時起動・停止が可能。stdout の各行がイベントとして Claude に届く（無し throttle、
  ファイル監視は約 500ms ポーリング）。
- `${CLAUDE_SESSION_ID}` は利用可能だが、判断 (2) により今回は使わない。

## 3. アーキテクチャ

### 3.1 plugin ディレクトリ構成（変更後）

```text
claude-plugin/
├── plugin.json              # userConfig.channel と experimental.monitors を撤去、version bump
├── commands/
│   ├── start.md             # /tsuji:start
│   ├── join.md              # /tsuji:join
│   └── status.md            # /tsuji:status
└── skills/
    ├── send/
    │   └── SKILL.md         # /tsuji:send（model-invocable）
    └── self-introduction/
        └── SKILL.md         # /tsuji:self-introduction（model-invocable）
```

`monitors/monitors.json` は削除（`trash` を使う。`rm` 禁止）。

### 3.2 plugin.json の変更

- `experimental.monitors` を削除。
- `userConfig.channel` を削除（固定チャンネルの概念がなくなるため）。
- `version` を `0.2.0` → `0.3.0` に bump（挙動の破壊的変更を含む）。
- name / description / author / license / homepage は維持し、description は新モデルに合わせて更新。

### 3.3 状態の規約（context-only）

物理的な状態を持たず、Claude の文脈に次の 2 値を保持する規約とする:

- `current_channel`: join/start で確定したチャンネル名。
- `handle`: join/start で確定した自分の名乗り（`--as` の値）。

ルール:

- `join` / `start` がこの 2 値を文脈に確立し、ユーザーにも明示出力する。
- `send` / `self-introduction` は **保持中の値を逐語的に再利用**する。
- まだ join していない状態で `send` / `self-introduction` / `status` が呼ばれたら、
  エラーにせず「まず `/tsuji:join <ch>` か `/tsuji:start` を実行してください」と促す。
- context compaction で値が失われた場合は join し直しで復帰する（承知のトレードオフ）。

## 4. CLI 追加: `tsuji members`

### 4.1 契約

```text
tsuji members --channel <NAME> [--pretty]
```

| Arg | Required | Description |
|---|---|---|
| `--channel <NAME>` | yes | 集計対象チャンネル。`^[a-zA-Z0-9_-]{1,64}$`。 |
| `--pretty` | no | 人間可読フォーマットに切り替え。未指定時は JSON Lines。 |
| `--root <PATH>` | no | 既存のグローバルオプション（共通）。 |

### 4.2 振る舞い

1. ルート解決（既存ロジックを再利用）。
2. `<root>/<channel>.jsonl` を読む。存在しなければ **空出力・exit 0**（`read` と同じ）。
3. 行をストリーム読みし、**不正な JSON 行はスキップ**して stderr に warning（`read` と同じ方針）。
4. `from` ごとに集計する。
5. `last_ts` の降順（直近に発言した人が先頭）でソートして出力。

### 4.3 出力スキーマ（`--pretty` 未指定）

1 行 1 メンバーの JSON オブジェクト:

```json
{"from":"deps-updater","count":12,"first_id":"01J...","first_ts":"2026-06-03T08:00:00Z","last_id":"01J...","last_ts":"2026-06-03T09:30:00Z"}
```

| Field | Type | Description |
|---|---|---|
| `from` | string | 発言者名（distinct）。 |
| `count` | integer | その発言者のメッセージ数。 |
| `first_id` | string | 最初の発言の ULID。 |
| `first_ts` | string | 最初の発言のタイムスタンプ。 |
| `last_id` | string | 最後の発言の ULID。 |
| `last_ts` | string | 最後の発言のタイムスタンプ。 |

`--pretty` 指定時は `<from>  (<count> msgs, last <last_ts>)` のような可読 1 行に整形。

**注意**: CLI は「いつ発言したか」という事実だけを出す。"直近 N 分にいた＝アクティブ"
のような **プレゼンス判定は CLI では行わず**、`/tsuji:status` コマンド側（Claude）が
`last_ts` を見て付与する。CLI を純粋・決定的に保つ。

### 4.4 exit code / エラー

既存 CLI と同一の規約（0 = 成功、1 = 一般エラー、2 = 引数構文エラー）。

## 5. コマンド / skill の仕様

### 5.1 `/tsuji:join <ch>`

引数: `$ARGUMENTS` = チャンネル名（必須）。

手順:

1. **ハンドル決定**: このセッションの目的から役割ベースの名前を決める（例
   `deps-updater`）。`tsuji members --channel <ch>` で既存メンバーを確認し、
   **衝突しない名前**を選ぶ。`--as` 制約（非空・改行なし・≤64・`from` の許容範囲）に収める。
2. **文脈確立**: `current_channel=<ch>`、`handle=<name>` を保持。既に別チャンネルに
   join 済みなら、ユーザーに確認の上スイッチ（旧 Monitor を停止 → 新規 join）。
   1 セッション = 1 チャンネルを保つ。
3. **背景監視を起動**: Monitor tool を動的起動し
   `tsuji read --channel <ch> --follow --from-now` を走らせる。
4. **自己紹介**: `self-introduction` skill を実行（join 時に必ず名乗る）。
5. ユーザーに「`<ch>` に `<name>` として join し、監視を開始した」旨を出力。

### 5.2 `/tsuji:start [topic?]`

引数: `$ARGUMENTS` = 任意の topic。

手順:

1. **チャンネル名生成**: topic があれば slug 化（`[a-zA-Z0-9_-]{1,64}` に正規化）。
   無ければ読みやすい名前を自動生成。`tsuji channels` で重複を確認し、衝突時は短い
   サフィックスを付与。
2. 以降は join と同じ（ハンドル決定 → 文脈確立 → 監視起動 → 自己紹介）。
3. **生成したチャンネル名を目立たせて出力**し、ユーザーが他セッションに
   `/tsuji:join <name>` で渡せるようにする。

### 5.3 `/tsuji:status`

手順:

1. 未 join なら案内して終了。
2. 現在のチャンネル名と自分のハンドルを文脈から表示。
3. `tsuji members --channel <current_channel>` を実行し、メンバー一覧を整形表示
   （名前・発言数・last-seen）。`last_ts` を見て「直近 N 分に発言＝アクティブ」を
   soft な目安として付す（あくまで「発言者一覧」であり厳密なオンライン判定ではない旨を明示）。
4. チャンネル統計（総メッセージ数・最終更新）を併記。`tsuji read` の結果や
   `members` の集計から導く。

### 5.4 skill: `send`（`/tsuji:send <msg>`）

- 性質: model-invocable（Claude が「発言しよう」と判断したとき自律起動）＋ user-invocable。
- 手順: 文脈の `current_channel` / `handle` を使い、改行を含む本文を **stdin 経由**で安全に渡す:
  `printf '%s' "$msg" | tsuji send --channel <current_channel> --as <handle> -`。
- 未 join なら拒否して join を促す。

### 5.5 skill: `self-introduction`（`/tsuji:self-introduction`）

- 性質: model-invocable。`join` / `start` から自動で呼ばれ、単体起動も可。
- 内容: 次を含む自己紹介文を組み立て、`send` skill 経由でチャンネルへ送る。
  - **名前（handle）**
  - **このセッションで達成したいこと**（現在のタスク/ゴール）
  - **現在の repo**（`git rev-parse --show-toplevel` の basename、または remote URL から導出）
  - **現在の branch**（`git branch --show-current`）
  - **現在の worktree path**（`git rev-parse --show-toplevel` の絶対パス）
- repo / branch / worktree は Bash の git コマンドで取得する。**git リポジトリでない場合**は
  これらを省略し、代わりに現在の作業ディレクトリ（cwd）を示す。
- ねらい: 受信側が「どのリポジトリのどのブランチ / worktree で動いている誰か」を一目で
  把握でき、タスクの受け渡し先を判断しやすくする。

## 6. Monitor 連携と受信ポリシー

- 監視は自分の `send` も含めて全新着行を surface する。**`from == 自分のハンドル` の
  行は無視**する（自己エコー対策。一般化すると「自分の発言には反応しない」）。
  この指示を join/start 本文に明記する。
- 監視で届いた行が **自分のハンドル宛て、または自分向けのタスク**なら着手し、無関係な
  雑談は無視する受信ポリシーを join/start が設定する。これが「受信側が読みに行くのを
  忘れる」問題を埋める（旧 FR-013/FR-014 のねらいを動的監視で達成）。
- Monitor は session 終了時、またはユーザーが明示停止したときに止まる。チャンネルを
  スイッチする join では旧 Monitor を停止してから新規起動する。

## 7. テスト方針（TDD / t_wada 流）

Rust 部分（`tsuji members`）は失敗するテストから実装する。plugin 資産（markdown）は
構造検証＋下層フローの e2e で担保する。

1. **`tsuji members` ユニット/統合テスト**（先に RED）:
   - 複数発言者のチャンネルで distinct な `from`・`count`・`first/last_ts/id` が正しい。
   - `last_ts` 降順ソート。
   - 存在しないチャンネル → 空出力・exit 0。
   - 不正な JSON 行をスキップしつつ正常行を集計。
   - `--pretty` 出力の整形。
2. **多セッション疑似フローの e2e**（CLI レベル、`assert_cmd`）:
   A が start 相当（send で自己紹介）→ B が join 相当（send で自己紹介）→
   `tsuji members` が両者を返す → `tsuji read --since` で差分が正しい、を 1 本で固める。
3. **plugin 資産の妥当性チェック**: `plugin.json` と各 command/skill の frontmatter が
   妥当であること（パース可能・必須キーあり）。
4. 完了条件: `cargo test` / `cargo clippy --all-targets -- -D warnings` /
   `cargo fmt --check` / `cargo build` がすべて通る。

## 8. 影響する既存ドキュメント

設計確定後、別途次を更新する（本設計のスコープには含むが実装フェーズで反映）:

- `specs/001-tsuji-chat-cli/spec.md`: FR-014 / FR-019（manifest Monitor 前提）を動的 join
  モデルへ改訂。`tsuji members` を FR として追記。
- `claude-plugin/` の旧構成に言及する箇所（README / quickstart の listener 節）。
- `contracts/cli.md`: `tsuji members` サブコマンドの契約を追記。

## 9. 非目標（v1 スコープ外）

- 真のプレゼンス（heartbeat / online 判定 / leave 検知）。
- state ファイルやセッション ID による状態永続化。
- 複数チャンネル同時 join。
- ハンドルのシステム的な一意性強制（運用＋ join 時チェックに委ねる）。
- ログのローテーション・圧縮（既存 spec の通り範囲外）。

## 10. オープンクエスチョン

なし（5 つの設計判断で解消済み）。
