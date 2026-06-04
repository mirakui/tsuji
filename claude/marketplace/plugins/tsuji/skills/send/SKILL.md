---
name: send
description: Send a message to the tsuji channel this session has joined. Use whenever you want to speak, reply, or hand off a task to other Claude sessions in the current tsuji channel.
argument-hint: "<message>"
allowed-tools: Bash
---

Send a message to the tsuji channel this session is currently in.

Message to send: `$ARGUMENTS` (if empty, use the message you intend to send from the current context).

1. If you have NOT joined a tsuji channel in this session (no remembered current channel and handle), tell the user to run `/tsuji:join <channel>` or `/tsuji:start` first, and stop.

2. Send the message to your current channel as your handle. The body may contain newlines, so pass it on stdin and tell `tsuji send` to read the body from stdin with `--body -`:

   ```bash
   printf '%s' "<message>" | tsuji send --channel <current-channel> --as <handle> --body -
   ```

   Substitute the actual `<message>`, `<current-channel>`, and `<handle>`. The `--body -` flag (note: a bare `-` is NOT accepted and exits with code 2) makes `tsuji send` read the message body from stdin, which safely preserves newlines and any special characters. `tsuji send` prints nothing on success (exit 0); a non-zero exit means the send failed, so surface the stderr to the user.

3. Briefly confirm to the user that the message was sent.
