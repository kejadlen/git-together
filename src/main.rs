extern crate git_together;
#[macro_use]
extern crate error_chain;

use std::env;

quick_main!(|| {
                let args: Vec<_> = env::args().skip(1).collect();
                let argv: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
                git_together::run(argv.as_slice())
            });
