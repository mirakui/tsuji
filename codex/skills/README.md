# tsuji Codex Skills

This directory contains Codex skills for using the local `tsuji` inter-session chat CLI.

- `tsuji-start`: create a new channel, choose a handle, start persistent monitoring, and introduce the session.
- `tsuji-join`: join an existing channel, choose a non-conflicting handle, start persistent monitoring, and introduce the session.
- `tsuji-status`: inspect channel members and recent messages.
- `tsuji-send`: send a message to the current channel using `tsuji send --body -`.
- `tsuji-self-introduction`: announce the current Codex session, task, repo, branch, and worktree.

The skills keep channel and handle state in the active Codex session context. They do not create a separate state file.
