# Quickstart: tsuji

**Feature**: 001-tsuji-chat-cli

**Date**: 2026-05-22

実装完了後に手動で動作確認するための最小手順。受入テスト（`tests/`）が自動化する内容と等価だが、手で叩いてユーザ感覚を確かめるために残す。

## 前提

- Rust toolchain（stable 1.79+）。
- macOS または Linux。
- ローカルで動く Claude Code セッション 2 つ以上（人間ユーザは 1 名）。

## 1. ビルドとインストール

```sh
cargo install --path .
# または開発中は:
cargo run -- <subcommand> ...
```

`tsuji --version` で確認。

## 2. 最小フロー（User Story 1）

ターミナル 1（送信側 Claude 役）:

```sh
tsuji send --channel default --as agent-a --body "依存関係を最新化しておいて"
```

ターミナル 2（受信側 Claude 役）:

```sh
tsuji read --channel default
# stdout に JSON Lines が 1 行
# {"id":"01J...","ts":"2026-05-22T08:30:12Z","from":"agent-a","body":"依存関係を最新化しておいて"}

LAST_ID=$(tsuji read --channel default | tail -n1 | jq -r .id)

# 続きを差分受信
tsuji send --channel default --as agent-a --body "追加で lint も通しておいて"
tsuji read --channel default --since "$LAST_ID"
# 直前の追加分のみ出力されることを確認
```

## 3. チャンネル分離（User Story 2）

```sh
tsuji send --channel infra --as agent-a --body "deploy 確認お願い"
tsuji send --channel research --as agent-a --body "論文の要約お願い"

tsuji channels
# infra
# research

tsuji read --channel infra
# infra 宛のみが返ることを確認
```

## 4. 人間観察モード（User Story 4）

```sh
tsuji read --channel default --follow --pretty
# 別ターミナルから send したメッセージが 2 秒以内に表示されることを目で確認（SC-005）
# Ctrl-C で停止
```

## 5. 並行送信ストレステスト（SC-003 の手動再現）

```sh
# 2 つのシェルを開き、同時に下記をそれぞれ 50 回ループで実行
for i in $(seq 1 50); do
  tsuji send --channel stress --as session-a --body "msg-a-$i"
done

# もう一方では:
for i in $(seq 1 50); do
  tsuji send --channel stress --as session-b --body "msg-b-$i"
done

# 完了後:
wc -l ~/.local/share/tsuji/stress.jsonl   # → 100
jq -c '.' ~/.local/share/tsuji/stress.jsonl > /dev/null  # 0 件の parse error
```

## 6. Claude plugin の Monitor による listener（FR-014）

```text
# 受信側の Claude Code セッション内で:
/plugin install <path-or-marketplace-id>   # claude-plugin/ ディレクトリを plugin として登録
# 初回 install で user_config.channel を求められるので、listen するチャンネル名を入力
# （未入力なら default = "default"）

# plugin が有効な状態でセッションを開始すると、Monitor tool が
#   tsuji read --channel <user_config.channel> --follow --from-now
# をバックグラウンドで起動する。/tasks や /plugin 画面で生きていることを確認できる。

# 別シェルから送信
tsuji send --channel default --as outsider --body 'are you awake?'

# Monitor が新着 JSON Lines を受信側 Claude のセッションに surface し、
# Claude が応答できる。過去ログ（--from-now）は流れないのでコンテキストが汚れない。
```

CLI 単体の挙動は `tsuji read --channel <ch> --follow --from-now` を任意のターミナルで起動して再現できる。

## 7. ロールバック

設定や状態は `<root>/<channel>.jsonl` ファイルのみ。捨てるには:

```sh
trash ~/.local/share/tsuji/  # global rule: rm 禁止
```

## 期待される SC 達成

- SC-001: 上記 2 のフローが手作業介入ゼロで完了する。
- SC-002: 各コマンドが 1 秒以内に応答。
- SC-003: 5 の手動ストレステストでログ破損ゼロ。
- SC-004: `--since` で過去メッセージが混入しない。
- SC-005: `--follow` が 2 秒以内に新着を表示。
- SC-006: `tsuji --help` で `send` / `read` の用法に 1 分以内で到達。
