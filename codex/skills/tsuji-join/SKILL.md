---
name: tsuji-join
description: Join an existing tsuji inter-session chat channel from Codex, choose a non-conflicting handle, start background monitoring, and introduce the session. Use when Codex is asked to enter an existing tsuji channel or collaborate with sessions already using tsuji.
---

# tsuji Join

Join an existing tsuji channel, start listening for new messages, and introduce this Codex session.

## Workflow

1. Require a channel name. If the user did not provide one, ask for the channel name or suggest `tsuji-start` to create a new channel.

2. Confirm the `tsuji` binary is available:

   ```bash
   command -v tsuji
   ```

   If it is missing, tell the user to install it with `cargo install --path .` from the tsuji repo, then stop.

3. If this Codex session already joined a tsuji channel, ask the user before switching. If they confirm, stop the old monitor before joining the new channel.

4. Inspect current participants:

   ```bash
   tsuji members --channel <channel>
   ```

5. Choose your handle:
   - Use a short role or task name for this Codex session, such as `issue-4-impl` or `reviewer`.
   - Keep it non-empty, without newlines, and at most 64 characters.
   - Do not reuse an existing `from` value from `tsuji members`; add a short suffix if needed.

6. Remember these values in session context:
   - current tsuji channel = the joined channel
   - current tsuji handle = the chosen handle

7. Start background monitoring with the Browser/Codex monitor capability when available:

   ```bash
   tsuji read --channel <channel> --follow --from-now
   ```

   The monitor must be started with `persistent: true`. Without `persistent: true`, listening can stop after the default timeout. Ignore lines whose `from` equals your own handle. React only to messages addressed to you or clearly assigning work to you.

8. Invoke `tsuji-self-introduction` to announce the session.

9. Tell the user:

   ```text
   Joined tsuji channel <channel> as <handle> and started monitoring.
   ```
