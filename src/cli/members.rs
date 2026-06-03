use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use ulid::Ulid;

use crate::message::Message;

/// One distinct sender in a channel, with activity bounds. Serialized as one
/// JSON object per line by `tsuji members`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MemberSummary {
    pub from: String,
    pub count: usize,
    pub first_id: Ulid,
    pub first_ts: DateTime<Utc>,
    pub last_id: Ulid,
    pub last_ts: DateTime<Utc>,
}

/// Aggregates messages into per-sender summaries, sorted most-recently-active
/// first (descending `last_id`; ULID order equals time order).
pub fn aggregate(messages: &[Message]) -> Vec<MemberSummary> {
    let mut map: HashMap<String, MemberSummary> = HashMap::new();
    for m in messages {
        map.entry(m.from.clone())
            .and_modify(|s| {
                s.count += 1;
                if m.id > s.last_id {
                    s.last_id = m.id;
                    s.last_ts = m.ts;
                }
                if m.id < s.first_id {
                    s.first_id = m.id;
                    s.first_ts = m.ts;
                }
            })
            .or_insert_with(|| MemberSummary {
                from: m.from.clone(),
                count: 1,
                first_id: m.id,
                first_ts: m.ts,
                last_id: m.id,
                last_ts: m.ts,
            });
    }
    let mut out: Vec<MemberSummary> = map.into_values().collect();
    out.sort_by(|a, b| b.last_id.cmp(&a.last_id));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn msg(id: &str, from: &str) -> Message {
        Message {
            id: Ulid::from_string(id).unwrap(),
            ts: chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            from: from.into(),
            body: "x".into(),
        }
    }

    #[test]
    fn aggregate_counts_first_and_last_per_sender() {
        let msgs = vec![
            msg("01ARZ3NDEKTSV4RRFFQ69G5F01", "a"),
            msg("01ARZ3NDEKTSV4RRFFQ69G5F02", "b"),
            msg("01ARZ3NDEKTSV4RRFFQ69G5F03", "a"),
        ];
        let got = aggregate(&msgs);
        assert_eq!(got.len(), 2);
        let a = got.iter().find(|m| m.from == "a").unwrap();
        assert_eq!(a.count, 2);
        assert_eq!(a.first_id.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5F01");
        assert_eq!(a.last_id.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5F03");
        let b = got.iter().find(|m| m.from == "b").unwrap();
        assert_eq!(b.count, 1);
    }

    #[test]
    fn aggregate_sorts_most_recently_active_first() {
        let msgs = vec![
            msg("01ARZ3NDEKTSV4RRFFQ69G5F01", "a"),
            msg("01ARZ3NDEKTSV4RRFFQ69G5F09", "b"),
        ];
        let got = aggregate(&msgs);
        assert_eq!(got[0].from, "b");
        assert_eq!(got[1].from, "a");
    }

    #[test]
    fn aggregate_empty_returns_empty() {
        assert!(aggregate(&[]).is_empty());
    }

    fn msg_at(id: &str, from: &str, hour: u32) -> Message {
        Message {
            id: Ulid::from_string(id).unwrap(),
            ts: chrono::Utc
                .with_ymd_and_hms(2026, 1, 1, hour, 0, 0)
                .unwrap(),
            from: from.into(),
            body: "x".into(),
        }
    }

    #[test]
    fn aggregate_tracks_first_and_last_ts() {
        use chrono::Timelike;
        let msgs = vec![
            msg_at("01ARZ3NDEKTSV4RRFFQ69G5F01", "a", 1),
            msg_at("01ARZ3NDEKTSV4RRFFQ69G5F03", "a", 3),
        ];
        let got = aggregate(&msgs);
        let a = &got[0];
        assert_eq!(a.first_ts.hour(), 1);
        assert_eq!(a.last_ts.hour(), 3);
    }
}
