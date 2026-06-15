---
name: tsuji-send
description: Send a message from Codex to the tsuji channel this session has joined. Use when Codex should reply, speak, hand off work, or broadcast status through the current tsuji inter-session chat.
---

# tsuji Send

Send a message to the tsuji channel this Codex session has joined.

## Workflow

1. Require session context:
   - current tsuji channel
   - current tsuji handle

   If either value is missing, tell the user to run `tsuji-join` with a channel or `tsuji-start` first, then stop.

2. Compose the message from the user request or current task context. Messages may contain newlines.

3. Send through stdin and `--body -`:

   ```bash
   printf '%s' "<message>" | tsuji send --channel <current-channel> --as <handle> --body -
   ```

   Always use `--body -` when reading the body from stdin. A bare trailing `-` is not accepted by the CLI and exits with code 2.

4. `tsuji send` prints nothing on success. If the command exits non-zero, surface stderr to the user. Otherwise, briefly confirm that the message was sent.
