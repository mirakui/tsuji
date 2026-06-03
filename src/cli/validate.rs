use crate::error::CliError;
use crate::message::id::is_valid_ulid_str;

pub fn validate_channel_name(s: &str) -> Result<(), CliError> {
    let len = s.len();
    if !(1..=64).contains(&len) {
        return Err(CliError::InvalidChannelName(s.to_string()));
    }
    if !s.bytes().all(is_channel_byte) {
        return Err(CliError::InvalidChannelName(s.to_string()));
    }
    Ok(())
}

const fn is_channel_byte(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-')
}

pub fn validate_sender(s: &str) -> Result<(), CliError> {
    if s.is_empty() || s.len() > 64 || s.contains('\n') {
        return Err(CliError::InvalidSender(s.to_string()));
    }
    Ok(())
}

pub fn validate_ulid_string(s: &str) -> Result<(), CliError> {
    if is_valid_ulid_str(s) {
        Ok(())
    } else {
        Err(CliError::InvalidUlid(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_channel_names() {
        for ok in ["default", "infra", "team-a", "team_b", "abc123"] {
            assert!(validate_channel_name(ok).is_ok(), "{ok} should pass");
        }
    }

    #[test]
    fn rejects_invalid_channel_names() {
        for bad in ["", "has space", "has!bang", "has/slash", "日本語"] {
            assert!(validate_channel_name(bad).is_err(), "{bad} should fail");
        }
    }

    #[test]
    fn rejects_too_long_channel_name() {
        let long = "a".repeat(65);
        assert!(validate_channel_name(&long).is_err());
    }

    #[test]
    fn accepts_normal_senders_and_rejects_newline_or_empty_or_too_long() {
        assert!(validate_sender("agent-a").is_ok());
        assert!(validate_sender("").is_err());
        assert!(validate_sender("a\nb").is_err());
        let long = "x".repeat(65);
        assert!(validate_sender(&long).is_err());
    }

    #[test]
    fn ulid_string_validation_aligns_with_id_module() {
        assert!(validate_ulid_string("01ARZ3NDEKTSV4RRFFQ69G5FAV").is_ok());
        assert!(validate_ulid_string("not-a-ulid").is_err());
    }
}
