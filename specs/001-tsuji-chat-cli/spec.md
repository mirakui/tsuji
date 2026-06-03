# Feature Specification: tsuji — Inter-Session Chat CLI for Claude Code

**Feature Branch**: `001-tsuji-chat-cli`

**Created**: 2026-05-22

**Status**: Draft

**Input**: User description: "tsuji（辻）— ローカルで起動中の複数の Claude Code セッション同士を会話させるための chat CLI。サーバは立てず、JSON Lines ファイルベースのチャットログを通じて、あるセッションから別セッションへ『これやっといて』とタスクを受け渡すような、ローカル完結のオーケストレーション用途を想定。Slack 風のチャンネル概念（1 jsonl = 1 チャンネル、参加者は名前で区別）。Claude Code からは Bash ツール経由で `tsuji send` / `tsuji read` などのサブコマンドを叩く。受信は Claude による自発的なポーリング（`tsuji read --since <last_id>`）。常駐 watch プロセスは持たない。人間は read-only で覗くだけで発言はしない。"

## Clarifications

### Session 2026-05-22

- Q: メッセージ ID の生成方式はどれにしますか？（FR-005 / Edge Cases『ID 重複』に対応） → A: ULID（26 文字、時刻＋ランダム、辞書順＝時系列順）
- Q: FR-013「受信側 Claude が tsuji read を叩くのを忘れる」課題への補助手段（Stop hook 連携等）は v1 でどう扱う？ → A: v1 に Claude skill を同梱して提供する。受信側 Claude が一度起動すれば、その skill が `/loop`（Claude Code のダイナミックループ／ScheduleWakeup）を使って自発的に再起動・ポーリングし続ける形にする。Stop hook ではなく `/loop` ベース。
- Q: チャンネルログの保存場所／検索規約はどうしますか？ → A: ユーザ global なディレクトリ（`$XDG_DATA_HOME/tsuji/`、無ければ `~/.local/share/tsuji/`）をデフォルトとし、`--root <path>` オプションおよび `TSUJI_ROOT` 環境変数による上書きを許す。
- Q: メッセージ本文の形式は v1 でどこまで許す？ → A: 改行を含むプレーンテキストのみ。本文は JSON 文字列としてエスケープ保存し、`read` 時に復元する。添付ファイル参照や構造化メタデータ（key-value）は v1 範囲外。
- Q: `tsuji read` の出力フォーマットは？ → A: デフォルトを JSON Lines（1 行 1 メッセージ、`id` / `ts` / `from` / `body` フィールド）とし、`--pretty` で人間可読フォーマットに切り替えられる。
- Q: FR-014 のリスナー実装は `/loop`（ScheduleWakeup）のままにするか？ → A: 切り替える。Claude Code v2.1.105+ の plugin-declared Monitor tool（`claude-plugin/monitors/monitors.json`）を採用し、プラグイン有効時にバックグラウンドで `tsuji read --channel <ch> --follow --from-now` を走らせ、新着行を Claude セッションに surface する。CLI 側にも `--from-now` を新設し、Monitor 起動時に過去ログを emit しない（FR-019）。`/loop` + `ScheduleWakeup` + skill 形式は廃止。FR-013（CLI 自体は daemon 同梱なし）はそのまま。

## User Scenarios & Testing *(mandatory)*

### User Story 1 - セッション A からセッション B へタスクを受け渡す (Priority: P1)

ユーザは別々のディレクトリ／コンテキストで複数の Claude Code セッションを同時に走らせている。セッション A の Claude が、現在のタスクから分岐した別作業（例: 「このリポジトリの依存関係を最新化しておいて」）をセッション B の Claude に依頼したい。A の Claude は Bash ツール経由で `tsuji send` を実行し、共有のチャンネルにメッセージを書き込む。B の Claude は自分の作業のキリの良いタイミングで `tsuji read --since <last_id>` を叩き、A からの依頼を読み取って実行に着手する。

**Why this priority**: これが本機能の存在意義そのもの。「セッション間でタスクを受け渡せる」という唯一のコア価値を実現する最小経路で、これが動かなければ他の機能はすべて無意味になる。

**Independent Test**: 同一マシン上で 2 つの Claude Code セッションを並行起動し、片方から `tsuji send` で書き込み、もう片方から `tsuji read` で受信できることを確認するだけで、機能としての価値が完結する。

**Acceptance Scenarios**:

1. **Given** チャンネル `default` が存在し、セッション A・B の双方から到達可能, **When** セッション A が `tsuji send --channel default --as agent-a "依存関係を更新しておいて"` を実行する, **Then** チャンネルログにメッセージが追記され、コマンドは非ゼロでない exit code（成功）を返す
2. **Given** 直前にセッション A がメッセージを送信した, **When** セッション B が `tsuji read --channel default --since <最後に読んだ ID>` を実行する, **Then** A の送ったメッセージ（送信者名・本文・タイムスタンプ・メッセージ ID を含む）が標準出力に出力され、それより前のメッセージは出力されない
3. **Given** セッション B が一度メッセージを読了済み, **When** 新規メッセージが無い状態で再度 `tsuji read --since <最新 ID>` を実行する, **Then** 新規メッセージなしの結果（空または明示的な「no new messages」シグナル）が返り、誤って既読分が再出力されない

---

### User Story 2 - チャンネルを使い分ける (Priority: P2)

ユーザは複数のタスク群を扱っており、用途別にチャンネルを分けたい（例: `infra`、`frontend`、`research`）。各セッションは自分が参加するチャンネルだけを読む。新しいチャンネルは、誰かが最初にそのチャンネル名で `tsuji send` した瞬間に自動的に作成される（明示的な作成コマンドは不要、もしくは別途用意）。

**Why this priority**: タスクの混線を避ける運用上の必須機能だが、最小 MVP（User Story 1）はチャンネル 1 本でも成立する。

**Independent Test**: 異なるチャンネル名 (`infra`, `research`) で送信されたメッセージが、`tsuji read --channel <name>` で各々独立に取得でき、他チャンネルのメッセージが混ざらないことを確認する。

**Acceptance Scenarios**:

1. **Given** チャンネル `infra` と `research` がそれぞれ存在する, **When** `tsuji read --channel infra` を実行する, **Then** `infra` 宛のメッセージのみが出力され、`research` のメッセージは出力されない
2. **Given** これまで存在しなかったチャンネル名 `newtopic`, **When** `tsuji send --channel newtopic --as agent-a "始めます"` を実行する, **Then** 新規チャンネルが自動的に作成され、その後 `tsuji read --channel newtopic` でメッセージが取得できる
3. **Given** 複数のチャンネルが存在する, **When** `tsuji channels`（または相当のサブコマンド）を実行する, **Then** 既存チャンネル名の一覧が出力される

---

### User Story 3 - 既読カーソルによる差分受信 (Priority: P2)

セッション B は自分が最後に読んだメッセージ ID（カーソル）を覚えておき、次回 `tsuji read --since <id>` を叩くと、その後に追加されたメッセージだけを受け取る。Claude Code の文脈上、毎回過去ログ全部を読まされてはコンテキストが汚れるため、差分受信が必須。

**Why this priority**: コンテキスト効率の観点で実用上ほぼ必須だが、`tsuji read` の素の挙動（全件返す）でも User Story 1 は技術的には成立する。

**Independent Test**: 任意のメッセージ ID を `--since` に渡したとき、それより新しいメッセージのみが返ることを単独テストで検証可能。

**Acceptance Scenarios**:

1. **Given** チャンネルに 10 件のメッセージがあり、5 件目までのメッセージ ID を `LAST` とする, **When** `tsuji read --channel default --since LAST` を実行する, **Then** 6 件目以降の 5 件のみが出力される
2. **Given** `--since` に存在しないメッセージ ID を渡す, **When** コマンドを実行する, **Then** エラー（または定義済みの安全なフォールバック）が返り、誤って全件出力されることはない
3. **Given** `--since` を省略する, **When** `tsuji read` を実行する, **Then** デフォルトの挙動（例: 直近 N 件、または全件）が文書化された通りに動作する

---

### User Story 4 - 人間による read-only 観察 (Priority: P3)

ユーザ自身（人間）は会話には参加せず、各セッションが何をやり取りしているか覗き見たい。`tsuji read --channel <name> --follow`（あるいは `tail -f` 相当）で、新規メッセージをリアルタイムに流し読みできる。書き込みは Claude セッションのみに任せる運用前提。

**Why this priority**: 開発・デバッグ時のオブザーバビリティに有用だが、機能としては「ファイルを直接 `tail -f` する」だけでも代替可能なので優先度は下げられる。

**Independent Test**: チャンネル `default` を `--follow` モードで開いた状態で別セッションから `tsuji send` を実行し、観察側にメッセージが追記表示されることを確認する。

**Acceptance Scenarios**:

1. **Given** ユーザがターミナルで `tsuji read --channel default --follow` を実行している, **When** 別セッションが同じチャンネルに `tsuji send` する, **Then** ユーザの画面にそのメッセージが（極端な遅延なく）表示される
2. **Given** ユーザの観察セッション, **When** 観察者が何らかの方法で書き込もうとしても（または書き込み手段が無く）、**Then** 観察モードでは送信機能が露出されない（read-only である）

---

### Edge Cases

- **同時書き込み**: 2 つのセッションがほぼ同時に同じチャンネルに `tsuji send` した場合、JSON Lines ファイルへの append が破損しない（行が混ざらない／途中で切れない）。
- **不正な JSON 行**: 何らかの理由でログファイルに不正な行が混入した場合、`tsuji read` は壊れた行をスキップしつつ正常な行は出力する（クラッシュしない）。
- **メッセージ ID の重複**: 同時送信時に同一 ID が生成される事故が起きないこと。ID 形式は ULID（26 文字、時刻＋ランダム）を採用し、辞書順＝時系列順となる性質を `--since` のカーソル比較に利用する。
- **巨大ログ**: チャンネルログが何 MB／何万行と肥大化した場合の `tsuji read --since` の応答時間が許容範囲に留まる（ローテーション・圧縮の方針は v1 範囲外、未決事項として記録）。
- **受信側の取りこぼし**: ポーリング型のため、受信側 Claude が `tsuji read` を叩くのを忘れるとメッセージが永遠に届かない。これに対する補助として、v1 では Claude Code plugin として `tsuji read --follow --from-now` を Monitor tool 経由で自動起動する（FR-014）。Monitor は同梱 plugin のマニフェスト宣言で session 開始時に起動するため、ユーザ／Claude が起動忘れを起こす経路を持たない。（v0.3 改定: 自動起動は manifest 宣言ではなく、/tsuji:join・/tsuji:start 実行時に Monitor を動的起動する形へ変更。channel と自分のハンドルはセッションの文脈で保持する。FR-014 参照。）
- **存在しないチャンネル読み込み**: まだ誰も書いたことのないチャンネル名で `tsuji read` を叩いた場合、エラーではなく「メッセージ無し」として安全に応答する。
- **`--as <name>` 衝突**: 同じセッションが複数の名前を名乗ったり、別セッションが同じ名前を名乗ったりすることを技術的に禁止はしない（名前は単なるラベル）。整合性はユーザ運用に委ねる。

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST メッセージを JSON Lines 形式のファイルに append-only で記録する。1 ファイル = 1 チャンネルに対応させる。
- **FR-002**: System MUST `tsuji send --channel <name> --as <sender> <body>` 相当のサブコマンドにより、指定チャンネルへメッセージを送信できる。
- **FR-003**: System MUST `tsuji read --channel <name>` で当該チャンネルのメッセージを時系列順に、デフォルトで JSON Lines 形式（1 行 1 メッセージ、`id` / `ts` / `from` / `body` フィールドを少なくとも含む）で標準出力に出力できる。
- **FR-004**: System MUST `tsuji read --channel <name> --since <message_id>` で指定 ID より後のメッセージのみを差分出力できる。出力形式は FR-003 と同一（デフォルト JSON Lines）。
- **FR-005**: System MUST 各メッセージに一意のメッセージ ID（ULID、26 文字）とタイムスタンプを付与し、受信者がカーソルとして使える形で出力する。ULID は辞書順＝時系列順を満たすため、`--since` のカーソル比較は文字列の辞書順で実装してよい。
- **FR-006**: System MUST 既存しないチャンネルに対する `send` で、当該チャンネルを自動的に新規作成する（または等価な明示作成手順を 1 ステップで提供する）。
- **FR-007**: System MUST 同一チャンネルへの並行 `send` においてログファイルの破損（行の混在・切断）を起こさない（atomic append を保証する）。
- **FR-008**: System MUST チャンネル名の一覧を取得する手段を提供する（例: `tsuji channels`、または所定ディレクトリの ls 相当）。
- **FR-009**: System MUST サーバ・常駐デーモンを必要としない。すなわち、`send` も `read` もワンショットコマンドとして完結する。
- **FR-010**: System MUST 人間ユーザが容易に追従できる手段を提供する。具体的には `tsuji read --follow [--pretty]`（および/または生ログファイルへの `tail -f`）で新規メッセージを継続的に観察できること。
- **FR-011**: System MUST 認証・権限管理は持たない（同一ユーザ・同一マシン上のローカル運用前提で、信頼境界はファイルシステム権限に委ねる）。
- **FR-012**: System SHOULD `tsuji send` / `tsuji read` の exit code および stderr で、Claude Code が成功・失敗を判定できるシグナルを返す。
- **FR-013**: System MUST 受信側が「思い出して読みに行く」ポーリングモデルを基本とし、watch 常駐プロセスは提供しない。
- **FR-014**: System MUST 受信側 Claude が新着メッセージを継続的に取得するための補助として、v1 に Claude Code plugin（`claude-plugin/`）を同梱する。plugin は `monitors/monitors.json` で Monitor tool を宣言し、セッション開始時に `tsuji read --channel <user_config.channel> --follow --from-now` をバックグラウンドで起動して、新着行を Claude セッションに surface する。`/loop` や `ScheduleWakeup`、Stop hook 連携は採用しない。（v0.3 改定: 固定チャンネルの manifest Monitor は廃止。listen は plugin の /tsuji:join・/tsuji:start が実行時に Monitor tool を動的起動して開始する。channel と自分のハンドルはセッションの文脈で保持する。）
- **FR-015**: System MUST チャンネルログの保存ルートを、デフォルトで `$XDG_DATA_HOME/tsuji/`（未設定時は `~/.local/share/tsuji/`）に置く。`--root <path>` コマンドラインオプションおよび `TSUJI_ROOT` 環境変数を提供し、その値で上書きできる（優先度: コマンドライン > 環境変数 > デフォルト）。
- **FR-016**: System MUST 指定された保存ルート配下に存在しないチャンネルへの `send` 時、必要なディレクトリ・ファイルを自動的に作成する（保存ルート自体が存在しない場合も含む）。
- **FR-017**: System MUST メッセージ本文として改行を含む任意長のプレーンテキストを受理する。本文は JSON Lines の 1 行に収まるように JSON 文字列としてエスケープ保存し、`read` 時には改行を含む元のテキストへ復元して出力する。添付ファイル参照や構造化メタデータ（key-value 拡張フィールド）は v1 では受理しない。
- **FR-018**: System MUST `tsuji read` の出力形式を切り替える `--pretty` フラグを提供する。`--pretty` 指定時は人間可読フォーマット（少なくともタイムスタンプ・送信者・本文を可読に並べた形）で出力し、未指定時は JSON Lines を出力する。
- **FR-019**: System MUST `tsuji read --follow` と併用可能な `--from-now` フラグを提供する。`--from-now` 指定時は実行開始時点の末尾までを cursor として確定し、既存メッセージを stdout に emit せず、それ以降に追加された新着メッセージのみを emit する。`--follow` 抜きで `--from-now` を指定した場合は CLI が引数エラー（exit code 非ゼロ）で拒否する。Monitor tool（FR-014）が過去ログをセッションに流し込まないために用いる。（--from-now は引き続き Monitor 起動時に過去ログを emit しないために使う。起動主体が manifest から /tsuji:join・/tsuji:start に変わった点のみ改定。）
- **FR-020**: System MUST `tsuji members --channel <name>` で当該チャンネルの発言者
  （distinct な `from`）を集計し、各人の発言数・最初/最後の発言 ID とタイムスタンプを
  `last_id` 降順の JSON Lines（`--pretty` で人間可読）で出力する。これはプレゼンス機構
  ではなく「発言履歴からの導出」であり、`/tsuji:status` のメンバー一覧の基盤となる。

### Key Entities *(include if feature involves data)*

- **Channel**: 1 本の JSON Lines ファイルに対応する論理的なチャット空間。識別子はチャンネル名（短い英数字想定）。
- **Message**: チャンネルログの 1 行に対応する不可変イベント。属性: `id`（ULID）、`ts`（タイムスタンプ）、`from`（送信者の自称名）、`body`（改行を含み得るプレーンテキスト本文）。
- **Sender (Participant)**: メッセージの `from` 値で識別される論理的な発話主体。物理的なセッションとの紐付けはユーザ運用に委ね、システムは強制しない。
- **Cursor**: 受信側が「ここまで読んだ」状態を表すメッセージ ID。クライアント（呼び出し側 Claude）が保持し、システムは保持しない（ステートレス）。

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 2 つの Claude Code セッション間で「タスクを依頼 → 反対側で読み取り → 実行に着手」までを、ユーザの手作業介入ゼロで完了できる。
- **SC-002**: `tsuji send` および `tsuji read` のコールドスタートから結果出力までの所要時間が、典型ケース（チャンネルあたり数百〜数千メッセージ）で 1 秒未満に収まる。
- **SC-003**: 同一チャンネルへの並行 `send`（2 セッション同時）を 100 回連続で実施しても、ログファイルの破損・行の混在が 0 件である。
- **SC-004**: 受信側が `--since <last_id>` を指定したとき、それより古いメッセージが返ってくる事象が 0 件である（カーソル意味論の完全性）。
- **SC-005**: 人間ユーザが任意のチャンネルを `--follow` （あるいは `tail -f`）で観察した際、新規メッセージが 2 秒以内に表示される。
- **SC-006**: 新規ユーザ（＝自分）が README なしでも `tsuji --help` から `send` / `read` の使い方に 1 分以内で到達できる（CLI の自己文書化度合いの指標）。

## Assumptions

- 利用者は 1 名（プロジェクト発案者本人）。マルチユーザ・OSS 配布は本仕様の対象外（将来検討）。
- 全セッションが同一マシンの同一ユーザ権限下で動作し、共通のディレクトリに書き込み可能である。デフォルトは `$XDG_DATA_HOME/tsuji/`（無ければ `~/.local/share/tsuji/`）、必要に応じて `--root` または `TSUJI_ROOT` で上書きする（FR-015）。
- 同一ファイルシステム上の append が atomic であることを前提とする（POSIX 準拠の通常ファイル、`O_APPEND` での書き込みを想定）。ネットワーク FS は対象外。
- 暗号化・認証は不要（ローカルプロセス間通信扱い）。機密情報をやり取りしないわけではないが、信頼境界は OS のファイル権限に委ねる。
- メッセージ本文はプレーンテキスト（改行を含んでよい）。添付・画像・構造化ペイロードは v1 範囲外（FR-017）。
- 過去ログのローテーション・圧縮は v1 範囲外。チャンネルログが肥大化したら手動 rotate する運用で許容する。
- DM（1 対 1 専用チャンネル）の特別扱いは v1 範囲外。必要なら専用名のチャンネル（例: `dm-foo-bar`）を作る運用で代替する。
- 受信側がポーリングを「思い出す」課題への自動的な解決（Stop hook 連携等）は v1 の必須機能ではなく、試作しながら必要性を判断する別レーンの検討事項とする。
- メッセージ ID は ULID を採用する（時刻ベース 26 文字、辞書順＝時系列順）。チャンネル内一意性および順序保証はこの性質に依存する。
