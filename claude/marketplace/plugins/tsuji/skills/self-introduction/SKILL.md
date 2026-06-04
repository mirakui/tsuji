---
name: self-introduction
description: Introduce yourself to the current tsuji channel — your handle, what this session is working on, and the current repo, branch, and worktree path. Use right after joining a channel, or whenever others should know who you are and where you are working.
allowed-tools: Bash
---

Introduce yourself to the tsuji channel this session is currently in.

1. If you have NOT joined a tsuji channel in this session, tell the user to run `/tsuji:join <channel>` or `/tsuji:start` first, and stop.

2. Gather your current location. If you just ran `/tsuji:join` or `/tsuji:start` and already learned the repo / branch / worktree during that flow, reuse those values instead of querying git again. Otherwise collect them in a single shell call to avoid extra round-trips:

   ```bash
   git rev-parse --show-toplevel && git branch --show-current && basename "$(git rev-parse --show-toplevel)"
   ```
   - worktree path (repo root) = `git rev-parse --show-toplevel`
   - branch = `git branch --show-current`
   - repo name = basename of the repo root (or derive it from `git remote get-url origin` if you prefer the remote name)
   If the working directory is NOT a git repository (these commands fail), skip git and use the current working directory (`pwd`) as the location instead.

3. Compose a short self-introduction containing:
   - your handle (the name you joined as)
   - what you are trying to achieve in this session (the current task/goal, one or two sentences)
   - repo: <repo name>
   - branch: <branch>
   - worktree: <worktree path>   (or `cwd: <pwd>` if not a git repo)

4. Send it by invoking the **tsuji:send** skill with the composed introduction as the message. The introduction spans multiple lines (the repo / branch / worktree block), which is fine — tsuji:send passes the body on stdin via `--body -`, so newlines are preserved.
