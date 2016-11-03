#![feature(advanced_slice_patterns, slice_patterns)]

extern crate git_together;

use std::env;
use std::process::Command;
use git_together::GitTogether;
use git_together::config::GitConfig;

fn main() {
  let config = GitConfig { namespace: "git-together".into() };
  let gt = GitTogether { config: config };

  let all_args: Vec<_> = env::args().skip(1).collect();
  let args: Vec<&str> = all_args.iter().map(String::as_ref).collect();

  match &args[..] {
    &["with", ref inits..] => {
      gt.set_authors(inits).unwrap();
    }
    &[sub_cmd, ref rest..] if sub_cmd == "commit" => {
      let mut git_cmd = Command::new("git");
      let cmd = git_cmd.arg(sub_cmd).args(rest);
      let signoff = gt.add_signoff(cmd).unwrap();
      signoff.status().unwrap();
    }
    x => {
      println!("{:?}", x);
    }
  }
}
