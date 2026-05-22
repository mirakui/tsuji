use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};

use crate::message::Message;
use crate::storage::paths::channel_path;

/// Reads all messages from a channel in append order.
///
/// Returns an empty `Vec` if the channel file does not yet exist. Malformed
/// lines are skipped with a warning printed to stderr.
pub fn read_messages(root: &Path, channel: &str) -> Result<Vec<Message>> {
    let path = channel_path(root, channel);
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("opening {}", path.display())),
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("reading {} line {}", path.display(), i + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Message>(&line) {
            Ok(m) => out.push(m),
            Err(e) => eprintln!(
                "tsuji: warning: skipping malformed line {} in {}: {}",
                i + 1,
                path.display(),
                e
            ),
        }
    }
    Ok(out)
}

/// Returns messages with id strictly greater than `since` (lexicographic).
pub fn filter_since(messages: Vec<Message>, since: Option<&str>) -> Vec<Message> {
    match since {
        None => messages,
        Some(s) => messages
            .into_iter()
            .filter(|m| m.id.to_string().as_str() > s)
            .collect(),
    }
}

/// Lists existing channel names by scanning `*.jsonl` files directly under `root`.
pub fn list_channels(root: &Path) -> Result<Vec<String>> {
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("reading {}", root.display())),
    };
    let mut names: Vec<String> = entries
        .filter_map(|res| res.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                return None;
            }
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(std::string::ToString::to_string)
        })
        .collect();
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::writer::append_message;

    #[test]
    fn returns_empty_for_missing_channel() {
        let dir = tempfile::tempdir().unwrap();
        let msgs = read_messages(dir.path(), "nope").unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn reads_back_appended_messages_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let m1 = Message::new("a", "one");
        let m2 = Message::new("a", "two");
        append_message(dir.path(), "ch", &m1).unwrap();
        append_message(dir.path(), "ch", &m2).unwrap();
        let got = read_messages(dir.path(), "ch").unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].body, "one");
        assert_eq!(got[1].body, "two");
    }

    #[test]
    fn filter_since_returns_only_newer() {
        use chrono::TimeZone;
        use ulid::Ulid;

        fn at(id: &str, body: &str) -> Message {
            Message {
                id: Ulid::from_string(id).unwrap(),
                ts: chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
                from: "a".into(),
                body: body.into(),
            }
        }

        let m1 = at("01ARZ3NDEKTSV4RRFFQ69G5F01", "one");
        let m2 = at("01ARZ3NDEKTSV4RRFFQ69G5F02", "two");
        let m3 = at("01ARZ3NDEKTSV4RRFFQ69G5F03", "three");
        let cursor = m2.id.to_string();
        let got = filter_since(vec![m1, m2, m3], Some(&cursor));
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].body, "three");
    }

    #[test]
    fn list_channels_returns_sorted_names() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["zeta", "alpha", "beta"] {
            append_message(dir.path(), name, &Message::new("a", "x")).unwrap();
        }
        let names = list_channels(dir.path()).unwrap();
        assert_eq!(names, vec!["alpha", "beta", "zeta"]);
    }

    #[test]
    fn list_channels_returns_empty_for_missing_root() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope");
        assert!(list_channels(&missing).unwrap().is_empty());
    }
}
