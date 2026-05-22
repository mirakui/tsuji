use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    Ok,
    Other(u8),
}

#[derive(Debug)]
pub enum CliError {
    InvalidChannelName(String),
    InvalidSender(String),
    EmptyBody,
    InvalidUlid(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChannelName(s) => write!(
                f,
                "channel: invalid name '{s}' (allowed: [a-zA-Z0-9_-]{{1,64}})"
            ),
            Self::InvalidSender(s) => write!(
                f,
                "as: invalid sender '{s}' (non-empty, no newline, length <= 64)"
            ),
            Self::EmptyBody => write!(f, "body: must not be empty"),
            Self::InvalidUlid(s) => write!(f, "since: not a valid ULID '{s}'"),
        }
    }
}

impl std::error::Error for CliError {}
