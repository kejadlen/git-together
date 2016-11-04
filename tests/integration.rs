extern crate git_together;
extern crate tempdir;

use std::env;
use std::process::Command;
use tempdir::TempDir;

fn main() {
  solo();
  pair();
}

fn solo() {
  let tmp_dir = setup();

  git_together(&["with", "jh"]);
  sh("touch foo");
  sh("git add foo");
  git_together(&["commit", "-m", "add foo"]);

  let author = sh("git show --no-patch --format=\"%aN <%aE>\"");
  assert_eq!(author, "James Holden <jholden@rocinante.com>");

  tmp_dir.close().unwrap();
}

fn pair() {
  let tmp_dir = setup();

  git_together(&["with", "jh", "nn"]);
  sh("touch foo");
  sh("git add foo");
  git_together(&["commit", "-m", "add foo"]);

  let author = sh("git show --no-patch --format=\"%aN <%aE>\"");
  assert_eq!(author, "James Holden <jholden@rocinante.com>");
  let committer = sh("git show --no-patch --format=\"%cN <%cE>\"");
  assert_eq!(committer, "Naomi Nagata <nnagata@rocinante.com>");

  tmp_dir.close().unwrap();
}

fn setup() -> TempDir {
  let tmp_dir = TempDir::new("git-together").expect("TempDir::new");
  env::set_current_dir(&tmp_dir).expect("env::set_current_dir");

  sh("git init");
  sh("git config --add git-together.domain rocinante.com");
  sh("git config --add git-together.authors.jh \"James Holden; jholden\"");
  sh("git config --add git-together.authors.nn \"Naomi Nagata; nnagata\"");
  sh("git config --add git-together.authors.ca \"Chrisjen Avasarala; avasarala@un.gov\"");

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
