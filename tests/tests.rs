use std::env;
use std::fs;
use std::process::Command;

#[test]
fn test() {
    let dir = env::temp_dir().join("git-together-tests");

    fs::create_dir_all(&dir).unwrap();

    let git = run("git", &["--version"]);
    let git_together = git_together();

    assert_eq!(git_together, git);
}

fn git_together() -> (i32, String) {
    let root = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let git_together = root.join("git-together");
    run(git_together.to_str().unwrap(), &["--version"])
}

fn run(cmd: &str, args: &[&str]) -> (i32, String) {
    let output = Command::new(cmd).args(args).output().unwrap();
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
    )
}
