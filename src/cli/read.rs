use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Args;

use crate::error::ExitCode;
use crate::message::Message;
use crate::pretty::pretty_format;
use crate::storage::reader::{filter_since, read_messages};

#[derive(Debug, Args)]
pub struct ReadArgs {
    /// Channel to read from.
    #[arg(long)]
    pub channel: String,

    /// Only return messages with id strictly greater than this ULID.
    #[arg(long, value_name = "ULID")]
    pub since: Option<String>,

    /// Output in human-readable format instead of JSON Lines.
    #[arg(long)]
    pub pretty: bool,

    /// After reading existing messages, keep polling for new ones.
    #[arg(long)]
    pub follow: bool,
}

pub fn run(root: &Path, args: ReadArgs) -> Result<ExitCode> {
    crate::cli::validate_channel_name(&args.channel)?;
    if let Some(s) = args.since.as_deref() {
        crate::cli::validate_ulid_string(s)?;
    }

    let mut cursor = args.since.clone();
    let messages = filter_since(read_messages(root, &args.channel)?, cursor.as_deref());

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for m in &messages {
        emit(&mut handle, m, args.pretty)?;
    }
    handle.flush()?;
    if let Some(last) = messages.last() {
        cursor = Some(last.id.to_string());
    }

    if !args.follow {
        return Ok(ExitCode::Ok);
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_handler = Arc::clone(&stop);
    let _ = ctrlc::set_handler(move || {
        stop_for_handler.store(true, Ordering::SeqCst);
    });

    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(500));
        let new_messages = filter_since(read_messages(root, &args.channel)?, cursor.as_deref());
        for m in &new_messages {
            emit(&mut handle, m, args.pretty)?;
        }
        handle.flush()?;
        if let Some(last) = new_messages.last() {
            cursor = Some(last.id.to_string());
        }
    }

    Ok(ExitCode::Ok)
}

fn emit<W: Write>(w: &mut W, m: &Message, pretty: bool) -> io::Result<()> {
    if pretty {
        writeln!(w, "{}", pretty_format(m))
    } else {
        let line = serde_json::to_string(m).expect("Message serialization must not fail");
        writeln!(w, "{line}")
    }
}
