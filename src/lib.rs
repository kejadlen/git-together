#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate git2;

mod errors;
use errors::*;

pub fn run(args: Vec<String>) -> Result<()> {
    GitTogether{}.run()
}

struct GitTogether {
}

impl GitTogether {
    fn run(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
}
