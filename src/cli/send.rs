use std::io::Read;
use std::path::Path;

use anyhow::{bail, Result};
use clap::Args;

use crate::error::ExitCode;
use crate::message::Message;
use crate::storage::writer::append_message;

#[derive(Debug, Args)]
pub struct SendArgs {
    /// Target channel.
    #[arg(long)]
    pub channel: String,

    /// Sender label.
    #[arg(long = "as", value_name = "SENDER")]
    pub from: String,

    /// Message body. Pass "-" to read from stdin.
    #[arg(long)]
    pub body: Option<String>,
}

pub fn run(root: &Path, args: SendArgs) -> Result<ExitCode> {
    crate::cli::validate_channel_name(&args.channel)?;
    crate::cli::validate_sender(&args.from)?;

    let body = match args.body.as_deref() {
        Some("-") => read_stdin()?,
        Some(text) => text.to_string(),
        None => bail!("body is required (use --body <TEXT> or --body -)"),
    };
    if body.is_empty() {
        return Err(crate::error::CliError::EmptyBody.into());
    }

    let msg = Message::new(args.from, body);
    append_message(root, &args.channel, &msg)?;
    Ok(ExitCode::Ok)
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    // Trim a single trailing newline that shells typically add.
    if buf.ends_with('\n') {
        buf.pop();
        if buf.ends_with('\r') {
            buf.pop();
        }
    }
    Ok(buf)
}
