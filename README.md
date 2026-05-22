# tsuji（辻）

Local file-based inter-session chat CLI for Claude Code.

ローカルで起動中の複数の Claude Code セッション同士を、サーバ無し・ファイルベースで会話させる小道具。1 セッションが別セッションに「これやっといて」とタスクを渡すのに使う。

## 仕組み（要約）

- ストレージは JSON Lines ファイル。1 ファイル＝1 チャンネル。
- メッセージ ID は ULID（辞書順＝時系列順）。
- 並行 send は `flock(2)` ＋ `O_APPEND` で破損ゼロ。
- 受信は `tsuji read --since <ULID>` の差分取得。常駐デーモンなし。
- 受信側 Claude のセッション開始時に `tsuji read --follow --from-now` をバックグラウンド起動する Claude Code plugin（`claude-plugin/` の `monitors/monitors.json`）を同梱。新着行は Monitor tool が surface するため、`/loop` や skill 起動忘れの心配がない。チャンネル名は plugin の `user_config.channel` で設定する。

## インストール

```sh
cargo install --path .
```

## クイックスタート

詳細は [specs/001-tsuji-chat-cli/quickstart.md](specs/001-tsuji-chat-cli/quickstart.md)。

```sh
# ターミナル A
tsuji send --channel default --as agent-a --body "依存関係を更新しておいて"

# ターミナル B
tsuji read --channel default                   # JSON Lines
tsuji read --channel default --pretty          # 人間可読
tsuji read --channel default --follow --pretty # tail -f 風

tsuji channels                                  # 既存チャンネル一覧
```

## 設定

| 項目 | 既定値 | 上書き |
|---|---|---|
| ログ保存ルート | `$XDG_DATA_HOME/tsuji/`（無ければ `~/.local/share/tsuji/`） | `--root <PATH>` または `TSUJI_ROOT` |

## 開発

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## License

MIT
