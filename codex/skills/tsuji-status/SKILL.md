---
name: tsuji-status
description: Inspect tsuji channel state from Codex, including known members and recent messages. Use when Codex needs to check who is present, what channel history exists, or whether the current tsuji collaboration channel is active.
---

# tsuji Status

Report useful status for the current or requested tsuji channel.

## Workflow

1. Determine the channel:
   - Prefer the current tsuji channel remembered in this Codex session.
   - If no channel is remembered, use the channel named by the user.
   - If neither is available, run `tsuji channels`, show the available channel names, and ask which one to inspect.

2. Show members:

   ```bash
   tsuji members --channel <channel> --pretty
   ```

   If pretty output is unavailable or unsuitable for automation, use JSON Lines without `--pretty`.

3. Show recent messages:

   ```bash
   tsuji read --channel <channel> --pretty
   ```

   Keep the user-facing summary concise. Include the latest messages, active handles, and any messages that appear addressed to this Codex session.

4. If the channel has no log or no members, say that the channel currently has no visible activity. Missing channels are not errors in tsuji; `read` and `members` may exit successfully with empty output.
