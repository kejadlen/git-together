#![feature(advanced_slice_patterns, slice_patterns)]

extern crate git_together;

use std::env;
use std::process::Command;

use git_together::GitTogether;
use git_together::author::AuthorParser;
use git_together::errors::*;
use git_together::git::{Config, GitConfig};

fn main() {
  run(|| {
    let all_args: Vec<_> = env::args().skip(1).collect();
    let args: Vec<&str> = all_args.iter().map(String::as_ref).collect();

    match args.as_slice() {
      &["with"] => {
        let mut gt = git_together()?;

        gt.set_active(&[])?;
        let authors = gt.all_authors()?;
        let mut sorted: Vec<_> = authors.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (initials, author) in sorted {
          println!("{}: {}", initials, author);
        }
      }
      &["with", ref inits..] => {
        let mut gt = git_together()?;

        let authors = gt.set_active(inits)?;
        for author in authors {
          println!("{}", author);
        }
      }
      &[sub_cmd, ref rest..] if ["commit", "merge", "revert"]
        .contains(&sub_cmd) => {
        let mut gt = git_together()?;

        if sub_cmd == "merge" {
          env::set_var("GIT_TOGETHER_NO_SIGNOFF", "1");
        }

        let mut cmd = Command::new("git");
        let cmd = cmd.arg(sub_cmd).args(rest);

        let signoff = gt.signoff(cmd)?;
        let status = signoff.status()
          .chain_err(|| "failed to execute process")?;
        if status.success() {
          gt.rotate_active()?;
        }
      }
      args => {
        Command::new("git").args(args)
          .status()
          .chain_err(|| "failed to execute process")?;
      }
    };

    Ok(())
  })
}

fn git_together() -> Result<GitTogether<GitConfig>> {
  let mut config = GitConfig::new("git-together")?;
  config.auto_include();

  let domain = config.get("domain")?;
  let author_parser = AuthorParser { domain: domain };

  Ok(GitTogether::new(config, author_parser))
}

fn run<F>(f: F)
  where F: Fn() -> Result<()>
{
  if let Err(error) = f() {
    for error in error.iter() {
      println!("{}", error);
    }
    std::process::exit(1);
  }
}
