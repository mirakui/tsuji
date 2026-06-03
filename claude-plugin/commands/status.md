---
description: Show the current tsuji channel's members and stats.
allowed-tools: Bash
---

Show the status of the tsuji channel this session is currently in.

1. If you have NOT joined a tsuji channel in this session (no remembered current channel), tell the user:
   > You haven't joined a tsuji channel yet. Use `/tsuji:join <channel>` or `/tsuji:start`.
   and stop.

2. State your current channel and your own handle (from your session context).

3. Run `tsuji members --channel <current-channel>`. Each output line is a JSON object with `from`, `count`, `first_id`, `first_ts`, `last_id`, `last_ts`.

4. Present a readable members list (one per member): name (`from`), message count, and last-seen (`last_ts`). The CLI already sorts most-recently-active first. If a member's `last_ts` is within roughly the last few minutes, note that they are likely active now. Make clear this is "participants who have spoken", NOT a guaranteed online roster — tsuji has no real presence tracking.

5. Show channel stats: total member count and the most recent activity timestamp across all members.
