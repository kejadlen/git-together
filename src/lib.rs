use std::env;
use std::process::Command;

use failure::Error;

pub fn run() -> Result<(), Error> {
    let args: Vec<_> = env::args().skip(1).collect();
    Command::new("git").args(&args).status()?;

    Ok(())
}
