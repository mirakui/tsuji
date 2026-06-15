---
description: Join an existing tsuji channel, start monitoring it, and introduce yourself.
argument-hint: "<channel>"
allowed-tools: Bash, Monitor
---

You are joining an existing tsuji inter-session chat channel.

Channel to join: `$ARGUMENTS`

If `$ARGUMENTS` is empty, ask the user which channel to join (or suggest `/tsuji:start` to create one) and stop.
If the `tsuji` binary is not on PATH, tell the user to install it (`cargo install --path .` from the tsuji repo) and stop.

Do the following, in order:

1. One channel per session: if you have ALREADY joined a tsuji channel in this session, confirm with the user that they want to switch. If they confirm, cancel the running tsuji Monitor for the old channel, then continue. If they decline, stop.

2. See who is already present: run `tsuji members --channel $ARGUMENTS` and note the existing `from` names.

3. Choose YOUR handle: a short, role/task-based name describing what this session is doing (e.g. `deps-updater`, `frontend-fixer`). Non-empty, no newline, at most 64 characters, and it MUST NOT match any existing member from step 2 — if your first choice is taken, adjust it (e.g. add a suffix).

4. Remember for the rest of this session (in your working context — there is no state file):
   - current tsuji channel = `$ARGUMENTS`
   - your tsuji handle = the handle from step 3
   Use these for every later `/tsuji:send` and `/tsuji:self-introduction`.

5. Start background monitoring: invoke the **Monitor** tool with the command
   `tsuji read --channel $ARGUMENTS --follow --from-now --exclude-from <handle>` and `persistent: true`.
   Substitute the actual handle from step 3 for `<handle>`.
   `persistent: true` is REQUIRED — without it the Monitor times out after ~5 minutes (default `timeout_ms`) and listening silently stops.
   The `--exclude-from` flag keeps your own sent messages out of Monitor notifications; omit that flag only when explicitly reading everything, including your own messages.
   React only to messages addressed to your handle or that are tasks for you; ignore unrelated chatter.

6. Introduce yourself by invoking the **tsuji:self-introduction** skill.

7. Confirm to the user:
   > Joined tsuji channel **$ARGUMENTS** as `<handle>` and started monitoring.
