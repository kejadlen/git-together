extern crate git_together;
extern crate tempdir;

use std::env;
use std::process::Command;
use tempdir::TempDir;

#[test]
fn solo() {
  let tmp_dir = setup();

  git_together(&["with", "jh"]);
  sh("touch foo");
  sh("git add foo");
  sh("git config --get-regexp git-together.*");
  git_together(&["commit", "-m", "add foo"]);

  let author = sh("git show --no-patch --format=\"%aN <%aE>\"");
  assert_eq!(author, "James Holden <jholden@rocinante.com>");

  tmp_dir.close().unwrap();
}

fn setup() -> TempDir {
  let tmp_dir = TempDir::new("git-together").expect("TempDir::new");
  env::set_current_dir(&tmp_dir).expect("env::set_current_dir");

  sh("git init");
  sh("git config --add git-together.domain rocinante.com");
  sh("git config --add git-together.authors.jh \"James Holden; jholden\"");

  tmp_dir
}

fn git_together(args: &[&str]) {
  let mut path = env::current_exe().unwrap();
  path.pop();

  let mut cmd = Command::new(path.join("git-together").to_str().unwrap());
  cmd.args(args).output().unwrap();
}

fn sh(s: &str) -> String {
  let output = Command::new("sh")
    .arg("-c")
    .arg(s)
    .output()
    .unwrap();
  String::from_utf8(output.stdout).unwrap().trim().into()
}
