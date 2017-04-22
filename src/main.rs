extern crate git_together;
#[macro_use]
extern crate error_chain;

use std::env;

quick_main!(|| {
    let args: Vec<String> = env::args().skip(1).collect();
    git_together::run(args)
});
