use std::cell::RefCell;

use ulid::{Generator, Ulid};

thread_local! {
    // Per-thread monotonic generator. Within a single thread, successive ULIDs
    // are strictly increasing even when produced within the same millisecond.
    // Cross-thread / cross-process sub-millisecond ordering is not guaranteed
    // (acceptable for personal-use traffic with flock-serialized writes).
    #[allow(clippy::missing_const_for_thread_local)]
    static GENERATOR: RefCell<Generator> = RefCell::new(Generator::new());
}

/// Generates a fresh ULID for the current moment, monotonic within this thread.
pub fn now_ulid() -> Ulid {
    GENERATOR.with(|g| g.borrow_mut().generate().unwrap_or_else(|_| Ulid::new()))
}

/// Returns true iff `s` is a syntactically valid 26-char Crockford-Base32 ULID.
pub fn is_valid_ulid_str(s: &str) -> bool {
    s.len() == 26 && s.bytes().all(is_crockford_base32_byte) && Ulid::from_string(s).is_ok()
}

const fn is_crockford_base32_byte(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'A'..=b'H' | b'J' | b'K' | b'M' | b'N' | b'P'..=b'T' | b'V'..=b'Z')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_consecutive_ulids_are_strictly_increasing_lexicographically() {
        let a = now_ulid().to_string();
        let b = now_ulid().to_string();
        assert!(a < b, "expected {a} < {b}");
    }

    #[test]
    fn ulid_strings_are_26_chars() {
        let s = now_ulid().to_string();
        assert_eq!(s.len(), 26);
    }

    #[test]
    fn validates_a_real_ulid() {
        let s = now_ulid().to_string();
        assert!(is_valid_ulid_str(&s), "{s} should validate");
    }

    #[test]
    fn rejects_malformed_strings() {
        assert!(!is_valid_ulid_str(""));
        assert!(!is_valid_ulid_str("not-a-ulid"));
        assert!(!is_valid_ulid_str("0000000000000000000000000I")); // contains forbidden 'I'
        assert!(!is_valid_ulid_str("0000000000000000000000000")); // 25 chars
    }
}
