---
name: tsuji-start
description: Create a new tsuji inter-session chat channel from Codex, choose a session handle, start background monitoring, and introduce the session. Use when Codex should open a fresh tsuji collaboration channel for other local agents or sessions to join.
---

# tsuji Start

Create a new tsuji channel, join it as this Codex session, start listening for new messages, and introduce yourself.

## Workflow

1. Confirm the `tsuji` binary is available:

   ```bash
   command -v tsuji
   ```

   If it is missing, tell the user to install it with `cargo install --path .` from the tsuji repo, then stop.

2. Decide the channel name:
   - If the user provided a topic, slugify it to `[a-zA-Z0-9_-]{1,64}` by lowercasing, replacing runs of other characters with `-`, trimming leading/trailing separators, and truncating to 64 characters.
   - If no topic was provided, choose a short readable name such as `room-sync` or `handoff-build`.
   - Run `tsuji channels` and ensure the name is not already listed. On collision, append a short suffix such as `-2`.

3. Choose your handle:
   - Use a short role or task name for this Codex session, such as `issue-4-impl` or `docs-writer`.
   - Keep it non-empty, without newlines, and at most 64 characters.

4. Remember these values in session context:
   - current tsuji channel = the chosen channel
   - current tsuji handle = the chosen handle

5. Start background monitoring with the Browser/Codex monitor capability when available:

   ```bash
   tsuji read --channel <channel> --follow --from-now
   ```

   The monitor must be started with `persistent: true`. Without `persistent: true`, listening can stop after the default timeout. Ignore lines whose `from` equals your own handle. React only to messages addressed to you or clearly assigning work to you.

6. Invoke `tsuji-self-introduction` to announce the session.

7. Tell the user:

   ```text
   Created and joined tsuji channel: <channel> as <handle>.
   Other Codex sessions can use $tsuji-join with channel <channel>.
   ```
