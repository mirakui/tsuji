---
name: tsuji-self-introduction
description: Introduce the current Codex session to a joined tsuji channel with its handle, task, repo, branch, and worktree. Use immediately after tsuji-start or tsuji-join, or whenever other tsuji participants need to know what this Codex session is doing.
---

# tsuji Self Introduction

Introduce this Codex session to the current tsuji channel.

## Workflow

1. Require session context:
   - current tsuji channel
   - current tsuji handle

   If either value is missing, tell the user to run `tsuji-join` with a channel or `tsuji-start` first, then stop.

2. Gather location context. Reuse values already collected by `tsuji-start` or `tsuji-join` if available. Otherwise run:

   ```bash
   git rev-parse --show-toplevel
   git branch --show-current
   ```

   If the current directory is not a git repo, use `pwd` as the location and omit branch/repo metadata.

3. Compose a short introduction with:
   - handle
   - current task or goal
   - repo name when available
   - branch when available
   - worktree path or cwd

4. Invoke `tsuji-send` with the introduction text. Multi-line introductions are expected; `tsuji-send` passes message bodies through stdin with `--body -`.
