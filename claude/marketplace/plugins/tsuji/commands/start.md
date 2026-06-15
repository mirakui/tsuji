---
description: Create a new tsuji channel, join it, start monitoring, and introduce yourself.
argument-hint: "[topic]"
allowed-tools: Bash, Monitor
---

You are joining the tsuji inter-session chat as the FIRST member of a brand-new channel.

Optional topic argument: `$ARGUMENTS`

If the `tsuji` binary is not on PATH, tell the user to install it (`cargo install --path .` from the tsuji repo) and stop.

Do the following, in order:

1. Decide a channel name:
   - If a topic was given in `$ARGUMENTS`, slugify it to match `[a-zA-Z0-9_-]{1,64}` (lowercase, runs of other characters become `-`, trim to 64 chars).
   - If no topic was given, invent a short, readable name (e.g. `room-amber`, `sync-otter`).
   - Run `tsuji channels` and confirm the name is NOT already listed. On collision, append a short suffix (e.g. `-2`, `-k3`) until unique.

2. Choose YOUR handle: a short, role/task-based name describing what this session is doing (e.g. `deps-updater`, `frontend-fixer`). Non-empty, no newline, at most 64 characters. (The channel is new, so no collision check is needed.)

3. Remember for the rest of this session (in your working context — there is no state file):
   - current tsuji channel = the name from step 1
   - your tsuji handle = the handle from step 2
   Use these for every later `/tsuji:send` and `/tsuji:self-introduction`.

4. Start background monitoring: invoke the **Monitor** tool with the command
   `tsuji read --channel <channel> --follow --from-now --exclude-from <handle>` and `persistent: true`.
   Substitute the actual channel from step 1 and handle from step 2.
   `persistent: true` is REQUIRED — without it the Monitor times out after ~5 minutes (default `timeout_ms`) and listening silently stops.
   The `--exclude-from` flag keeps your own sent messages out of Monitor notifications; omit that flag only when explicitly reading everything, including your own messages.
   React only to messages addressed to your handle or that are tasks for you; ignore unrelated chatter.

5. Introduce yourself by invoking the **tsuji:self-introduction** skill.

6. Tell the user, prominently, the channel name so they can pass it to other sessions:
   > Created and joined tsuji channel: **<channel>** (you are `<handle>`). Other sessions can join with `/tsuji:join <channel>`.
