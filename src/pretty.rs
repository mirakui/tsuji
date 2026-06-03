use crate::message::Message;

pub fn pretty_format(msg: &Message) -> String {
    let mut lines = msg.body.split('\n');
    let first = lines.next().unwrap_or("");
    let mut out = format!("[{}] {}: {}", msg.ts.to_rfc3339(), msg.from, first);
    for line in lines {
        out.push('\n');
        out.push_str("    ");
        out.push_str(line);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use ulid::Ulid;

    fn fixed_msg(from: &str, body: &str) -> Message {
        Message {
            id: Ulid::from_string("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap(),
            ts: chrono::Utc
                .with_ymd_and_hms(2026, 5, 22, 8, 30, 12)
                .unwrap(),
            from: from.into(),
            body: body.into(),
        }
    }

    #[test]
    fn single_line_body_formats_inline() {
        let m = fixed_msg("agent-a", "hello world");
        let s = pretty_format(&m);
        assert_eq!(s, "[2026-05-22T08:30:12+00:00] agent-a: hello world");
    }

    #[test]
    fn multi_line_body_indents_continuations() {
        let m = fixed_msg("agent-a", "first\nsecond\nthird");
        let s = pretty_format(&m);
        let mut lines = s.lines();
        assert_eq!(
            lines.next().unwrap(),
            "[2026-05-22T08:30:12+00:00] agent-a: first"
        );
        assert_eq!(lines.next().unwrap(), "    second");
        assert_eq!(lines.next().unwrap(), "    third");
    }
}
