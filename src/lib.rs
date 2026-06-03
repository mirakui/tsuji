//! tsuji — Local file-based inter-session chat CLI for Claude Code.
//!
//! Two or more Claude Code sessions running on the same machine talk to each
//! other by appending JSON Lines to a shared channel file. Receivers poll with
//! `tsuji read --since <last_id>` to fetch only new messages.

pub mod cli;
pub mod error;
pub mod message;
pub mod pretty;
pub mod storage;

use clap::Parser;

use crate::cli::Cli;
use crate::error::ExitCode;

/// Parses CLI arguments and dispatches to the appropriate subcommand handler.
///
/// Returns the process exit code that `main` should `std::process::exit` with.
pub fn run() -> i32 {
    let cli = Cli::parse();
    match cli::dispatch(cli) {
        Ok(ExitCode::Ok) => 0,
        Ok(ExitCode::Other(n)) => i32::from(n),
        Err(err) => {
            eprintln!("tsuji: {err:#}");
            1
        }
    }
}
