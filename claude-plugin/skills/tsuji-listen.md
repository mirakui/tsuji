---
name: tsuji-listen
description: Listen to a tsuji channel and surface new messages into the current Claude Code session, re-scheduling itself via `/loop` so it keeps polling without a long-running daemon.
arguments:
  - name: channel
    description: Channel name to listen on (e.g., "default", "infra"). Must match `[a-zA-Z0-9_-]{1,64}`.
    required: true
  - name: cursor_file
    description: Optional path (relative to repo root) where the last-seen ULID is stored. Defaults to `.cctmp/tsuji-cursor-<channel>`.
    required: false
---

# tsuji-listen

You are running the tsuji listener for channel **{{channel}}**.

## What you need to do this turn

1. Determine the cursor file path. If `{{cursor_file}}` is provided, use it. Otherwise default to `.cctmp/tsuji-cursor-{{channel}}`.
2. Read the cursor file if it exists; the contents (trimmed) are the last-seen ULID. If the file does not exist, treat the cursor as empty.
3. Run the tsuji CLI to fetch any new messages since the cursor:
   - If a cursor exists: `tsuji read --channel {{channel}} --since <cursor>`
   - Otherwise: `tsuji read --channel {{channel}}`
4. Parse the stdout as JSON Lines. For each line, surface the `from` and `body` fields to the current session (e.g., as a short summary or by acting on the request, depending on what the user asked you to do this session).
5. If at least one message was returned, write the highest (lexicographically last) `id` to the cursor file, creating parent directories as needed. Treat the JSON Lines output as already-ordered.
6. Schedule the next poll via `ScheduleWakeup` (i.e., the `/loop` mechanism):
   - `delaySeconds`: pick from 60 (busy channel) to 300 (idle). Default to 120 if unsure.
   - `prompt`: pass the same `/tsuji-listen {{channel}}` invocation back so the loop continues.
   - `reason`: one short sentence such as "watching tsuji channel {{channel}} for new tasks".
7. **Do not block.** Finish this turn after scheduling. The loop will fire again automatically.

## Stop conditions

- Stop scheduling the next wake-up if the user explicitly says "stop listening", "解除", or similar.
- Stop if `tsuji read` returns a non-zero exit status three turns in a row (treat as the binary being unavailable or the storage root misconfigured); print a one-line diagnostic to the user and exit the loop.

## Notes

- The tsuji CLI must be on PATH. If not, ask the user to install it (`cargo install --path .` from the tsuji repo).
- This skill is meant to run alongside other work in the session; respect ongoing user tasks and yield control quickly each turn.
