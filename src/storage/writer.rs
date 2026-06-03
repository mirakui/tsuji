use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use crate::message::Message;
use crate::storage::lock::ExclusiveLockGuard;
use crate::storage::paths::channel_path;

/// Appends `msg` as a single JSON Lines record to the channel file under `root`.
///
/// Creates the channel file and any missing parent directories. The write is
/// performed under an exclusive `flock(2)` lock on the channel file to ensure
/// concurrent senders cannot interleave bytes within a line.
pub fn append_message(root: &Path, channel: &str, msg: &Message) -> Result<()> {
    fs::create_dir_all(root)
        .with_context(|| format!("failed to create storage root {}", root.display()))?;
    let path = channel_path(root, channel);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open channel file {}", path.display()))?;

    let mut line = serde_json::to_string(msg).context("failed to serialize message")?;
    line.push('\n');

    let _guard = ExclusiveLockGuard::acquire(&file)?;
    (&file).write_all(line.as_bytes())?;
    (&file).flush()?;
    Ok(())
}
