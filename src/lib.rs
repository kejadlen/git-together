#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate git2;

mod errors;
use errors::*;

pub fn run() -> Result<()> {
    Ok(())
}
