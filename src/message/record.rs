use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// One immutable event in a channel log. Corresponds to a single line in the
/// underlying JSON Lines file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    /// ULID — 26 characters, lexicographically ordered = time-ordered.
    pub id: Ulid,
    /// Send-time timestamp (RFC3339, UTC).
    pub ts: DateTime<Utc>,
    /// Sender's self-declared label.
    pub from: String,
    /// Plain-text body. May contain newlines; serialized as a JSON string.
    pub body: String,
}

impl Message {
    /// Builds a new `Message` stamped with the current time and a fresh ULID.
    pub fn new(from: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: crate::message::id::now_ulid(),
            ts: Utc::now(),
            from: from.into(),
            body: body.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let m = Message::new("agent-a", "hello\nworld");
        let line = serde_json::to_string(&m).unwrap();
        let back: Message = serde_json::from_str(&line).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn json_contains_required_fields() {
        let m = Message::new("agent-a", "body");
        let line = serde_json::to_string(&m).unwrap();
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        for key in ["id", "ts", "from", "body"] {
            assert!(v.get(key).is_some(), "expected key {key} in {line}");
        }
    }

    #[test]
    fn id_is_serialized_as_26_char_string() {
        let m = Message::new("a", "b");
        let v: serde_json::Value = serde_json::to_value(&m).unwrap();
        let id = v.get("id").unwrap().as_str().unwrap();
        assert_eq!(id.len(), 26, "id was {id}");
    }

    #[test]
    fn body_preserves_newlines_across_serialize_roundtrip() {
        let body = "line1\nline2\n\nline4";
        let m = Message::new("a", body);
        let line = serde_json::to_string(&m).unwrap();
        assert!(!line.contains('\n'), "raw newline leaked: {line}");
        let back: Message = serde_json::from_str(&line).unwrap();
        assert_eq!(back.body, body);
    }
}
