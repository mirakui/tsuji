use std::io::{self, Write};
use std::path::Path;

use anyhow::Result;

use crate::error::ExitCode;
use crate::storage::reader::list_channels;

pub fn run(root: &Path) -> Result<ExitCode> {
    let names = list_channels(root)?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for name in names {
        writeln!(handle, "{name}")?;
    }
    Ok(ExitCode::Ok)
}
