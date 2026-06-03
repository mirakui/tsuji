# tsuji (辻)

Local file-based inter-session chat CLI for Claude Code.

A small tool that lets multiple Claude Code sessions on the same machine talk
to each other through a shared file — no server, no auth. One session hands
work off to another by saying "can you take this?".

## How it works

- Storage is a JSON Lines file per channel (1 file = 1 channel).
- Each message carries a 26-character ULID (lex order = time order).
- Concurrent sends are serialized with `flock(2)` + `O_APPEND`, so no line
  ever interleaves with another.
- Receivers fetch incrementally with `tsuji read --since <ULID>`. There is no
  long-running daemon in the CLI.
- A bundled Claude Code plugin (`claude-plugin/`, declared in
  `monitors/monitors.json`) starts `tsuji read --follow --from-now` in the
  background whenever a Claude Code session is active. New lines are
  delivered into the session by the Monitor tool, so there is no `/loop`
  rescheduling and no "did I forget to start the skill?" failure mode. The
  channel name is configured per install via `user_config.channel`.

## Install

```sh
cargo install --path .
```

## Quickstart

See [specs/001-tsuji-chat-cli/quickstart.md](specs/001-tsuji-chat-cli/quickstart.md) for details.

```sh
# Terminal A
tsuji send --channel default --as agent-a --body "please bump the dependencies"

# Terminal B
tsuji read --channel default                   # JSON Lines (default)
tsuji read --channel default --pretty          # human-readable
tsuji read --channel default --follow --pretty # tail -f-style

tsuji channels                                  # list existing channels
```

## Configuration

| Item | Default | Override |
|---|---|---|
| Channel log root | `$XDG_DATA_HOME/tsuji/` (falls back to `~/.local/share/tsuji/`) | `--root <PATH>` or `TSUJI_ROOT` |

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## License

MIT
