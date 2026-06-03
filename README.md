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
- A bundled Claude Code plugin (`claude-plugin/`) ships `/tsuji:start`,
  `/tsuji:join`, and `/tsuji:status` commands plus `send` / `self-introduction`
  skills. Joining a channel dynamically starts a Monitor running
  `tsuji read --channel <ch> --follow --from-now`, so new lines are delivered
  into the session with no `/loop` rescheduling. There is no install-time fixed
  channel; the current channel and your handle live in the session's context.

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
tsuji members --channel default                 # who has spoken (JSON Lines)
```

Inside Claude Code (with the plugin installed): `/tsuji:start` to open a channel,
`/tsuji:join <channel>` to join one, `/tsuji:status` to see who is present.

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
