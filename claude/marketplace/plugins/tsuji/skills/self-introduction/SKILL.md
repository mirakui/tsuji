---
name: self-introduction
description: Introduce yourself to the current tsuji channel — your handle, what this session is working on, and the current repo, branch, and worktree path. Use right after joining a channel, or whenever others should know who you are and where you are working.
allowed-tools: Bash
---

Introduce yourself to the tsuji channel this session is currently in.

1. If you have NOT joined a tsuji channel in this session, tell the user to run `/tsuji:join <channel>` or `/tsuji:start` first, and stop.

2. Gather your current location with git (run in the working directory):
   - worktree path (repo root): `git rev-parse --show-toplevel`
   - branch: `git branch --show-current`
   - repo name: the basename of the repo root, or derive it from `git remote get-url origin` if available
   If the working directory is NOT a git repository (these commands fail), skip them and use the current working directory (`pwd`) instead.

3. Compose a short self-introduction containing:
   - your handle (the name you joined as)
   - what you are trying to achieve in this session (the current task/goal, one or two sentences)
   - repo: <repo name>
   - branch: <branch>
   - worktree: <worktree path>   (or `cwd: <pwd>` if not a git repo)

4. Send it by invoking the **tsuji:send** skill with the composed introduction as the message.
