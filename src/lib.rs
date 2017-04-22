#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate git2;

use std::process;

mod errors;
use errors::*;

pub fn run(args: &[&str]) -> Result<()> {
    GitTogether::new().run(args)
}

struct GitTogether {
    quiet: bool,
}

impl GitTogether {
    fn new() -> Self {
        Self { quiet: false }
    }

    fn run(&self, args: &[&str]) -> Result<()> {
        let mut cmd = process::Command::new("git");
        cmd.args(args);
        if self.quiet {
            cmd.stdout(process::Stdio::null());
        }
        cmd.status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;
    use super::*;
    use std::env::set_current_dir;
    use std::process::{Command, Output};
    use self::tempdir::TempDir;

    #[test]
    fn passthrough_by_default() {
        let test = IntegrationTest::new();

        let gt = GitTogether { quiet: true };
        let args = ["commit", "-m", "ohai"];
        gt.run(&args).unwrap();

        assert_eq!(test.last_author(), "Original User <email@example.com>");
    }

    struct IntegrationTest {
        tmp_dir: TempDir,
    }

    impl IntegrationTest {
        fn new() -> Self {
            let tmp_dir = TempDir::new("example").unwrap();
            set_current_dir(tmp_dir.path()).unwrap();

            Self { tmp_dir }.tap(|it| {
                it.run(&["git", "init"]);
                it.run(&["git", "config", "--global", "user.name", "Original User"]);
                it.run(&["git",
                         "config",
                         "--global",
                         "user.email",
                         "email@example.com"]);
                it.run(&["touch", "foo"]);
                it.run(&["git", "add", "foo"]);
            })
        }

        fn tap<F: Fn(&Self) -> ()>(self, f: F) -> Self {
            f(&self);
            self
        }

        fn run(&self, cmd: &[&str]) -> Output {
            let base = &cmd[0];
            let args = &cmd[1..];
            Command::new(base)
                .args(args)
                .current_dir(self.tmp_dir.path())
                .output()
                .unwrap()
        }

        fn last_author(&self) -> String {
            let output = self.run(&["git", "show", "--format=%aN <%aE>", "--no-patch"]);
            String::from_utf8(output.stdout)
                .unwrap()
                .trim_right()
                .into()
        }
    }
}
