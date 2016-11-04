#![feature(advanced_slice_patterns, slice_patterns)]

extern crate git_together;

use std::env;
use std::process::Command;
use git_together::GitTogether;
use git_together::config::GitConfig;
use git_together::errors::*;

fn main() {
  run(|| {
    let config = GitConfig { namespace: "git-together".into() };
    let gt = GitTogether { config: config };

    let all_args: Vec<_> = env::args().skip(1).collect();
    let args: Vec<&str> = all_args.iter().map(String::as_ref).collect();

    match &args[..] {
      &["with", ref inits..] => {
        try!(gt.set_active(inits));
      }
      &[sub_cmd, ref rest..] if sub_cmd == "commit" => {
        let mut git_cmd = Command::new("git");
        let cmd = git_cmd.arg(sub_cmd).args(rest);
        let signoff = try!(gt.signoff(cmd));

        try!(signoff.status().chain_err(|| ""));
        try!(gt.rotate_active());
      }
      args => {
        try!(Command::new("git").args(args).status().chain_err(|| ""));
      }
    };

    Ok(())
  })
}

fn run<F>(f: F)
  where F: Fn() -> Result<()>
{
  if let Err(e) = f() {
    println!("{}", e);
    std::process::exit(1);
  }
}
