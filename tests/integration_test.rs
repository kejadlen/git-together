extern crate git_together;
extern crate tempdir;

use std::env;
use std::process::{Command, Output};
use tempdir::TempDir;

#[test]
fn it_works() {
  setup();

  let mut git_together = git_together_cmd();
  git_together.args(&["with", "jh"]).status().expect("git-together with jh");

  // sh("touch foo");
  // sh("git commit -m 'add foo'");
}

fn setup() {
  let tmp_dir = TempDir::new("repo").expect("TempDir::new");
  env::set_current_dir(&tmp_dir).expect("env::set_current_dir");

  sh("git init");
  sh("git config --add git-together.domain rocinante.com");
  sh("git config --add git-together.authors.jd \"James Holden; jholden\"");
}

fn git_together_cmd() -> Command {
  let mut path = env::current_exe().unwrap();
  path.pop();
  Command::new(path.join("git-together").to_str().unwrap())
}

fn sh(s: &str) -> Output {
  Command::new("sh").arg("-c").arg(s).output().unwrap()
}
