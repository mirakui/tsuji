pub mod channels;
pub mod members;
pub mod read;
pub mod send;
mod validate;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::error::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "tsuji",
    version,
    about = "Local file-based inter-session chat CLI"
)]
pub struct Cli {
    /// Override the channel storage root.
    #[arg(long, global = true, value_name = "PATH")]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Send a message to a channel.
    Send(send::SendArgs),
    /// Read messages from a channel.
    Read(read::ReadArgs),
    /// List existing channels.
    Channels,
}

pub fn dispatch(cli: Cli) -> Result<ExitCode> {
    let root = crate::storage::paths::resolve_root_from_env(cli.root.as_deref());
    match cli.command {
        Commands::Send(args) => send::run(&root, args),
        Commands::Read(args) => read::run(&root, args),
        Commands::Channels => channels::run(&root),
    }
}

pub use validate::{validate_channel_name, validate_sender, validate_ulid_string};
