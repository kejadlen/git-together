use std::env;
use std::process::{Command, Output};

#[test]
fn integration() {
    let mut path = env::current_dir().unwrap();
    path.push("target");
    path.push("debug");
    path.push("git-together");
    let git_together = path.to_str().unwrap();

    let temp_dir = env::temp_dir();
    let _ = env::set_current_dir(temp_dir);

    run("git", &["init"]);
    run("git", &["config", "--add", "git-together.domain", "rocinante.com"]);
    run("git", &["config", "--add", "git-together.authors.jh", "James Holden; jholden"]);
    run("git", &["config", "--add", "git-together.authors.nn", "Naomi Nagata; nnagata"]);
    run("git", &["config", "--add", "git-together.authors.ca", "Chrisjen Avasarala; avasarala@un.gov"]);

    // let output = run(git_together, &[]);
    // assert!(!output.status.success());

    run(git_together, &["with", "jh", "nn"]);
    run("touch", &["foo"]);
    run(git_together, &["add", "foo"]);
    run(git_together, &["commit", "-m", "added foo"]);

    let output = run("git", &["show", "--no-patch", "--format=\"%aN <%aE>\""]);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "\"James Holden <jholden@rocinante.com>\"\n");
    let output = run("git", &["show", "--no-patch", "--format=\"%cN <%cE>\""]);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "\"Naomi Nagata <nnagata@rocinante.com>\"\n");
    let output = run("git", &["show", "--no-patch", "--format=\"%B\""]);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "\"added foo\n\nSigned-off-by: Naomi Nagata <nnagata@rocinante.com>\n\"\n");
}

#[allow(unused_must_use)]
fn run(program: &str, args: &[&str]) -> Output {
    Command::new(program).args(args).output().unwrap()
}

