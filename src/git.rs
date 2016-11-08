use std::env;
use std::process::{Command, Output};
use git2;
use errors::*;

pub trait Config {
  fn get(&self, name: &str) -> Result<String>;
  fn set(&mut self, name: &str, value: &str) -> Result<()>;
}

pub struct GitConfig {
  namespace: String,
  repo: git2::Repository,
  config: git2::Config,
}

impl GitConfig {
  pub fn new(namespace: &str) -> Result<GitConfig> {
    let path = try!(env::current_dir().chain_err(|| ""));
    let repo = try!(git2::Repository::discover(path).chain_err(|| ""));
    let config = try!(repo.config().chain_err(|| ""));

    Ok(GitConfig {
      namespace: namespace.into(),
      repo: repo,
      config: config,
    })
  }

  pub fn auto_include(&self) {
    let filename = format!(".{}", self.namespace);
    let include_path = format!("../{}", filename);

    // Make sure .git-together exists
    if let Some(path) = self.repo.workdir() {
      let mut path_buf = path.to_path_buf();
      path_buf.push(&filename);
      if !path_buf.exists() {
        return;
      }
    } else {
      return;
    }

    // Make sure we're not already including .git-together
    if let Ok(output) = self.output(&["--local", "--get-all", "include.path"]) {
      let stdout = String::from_utf8_lossy(&output.stdout);
      if stdout.split('\n').any(|x| x == include_path) {
        return;
      }
    }

    let _ = self.output(&["--add", "include.path", &include_path]);
  }

  fn output(&self, args: &[&str]) -> Result<Output> {
    let output = try!(Command::new("git")
      .arg("config")
      .args(args)
      .output());

    if output.status.success() {
      Ok(output)
    } else {
      Err(ErrorKind::GitConfig(output).into())
    }
  }
}

impl Config for GitConfig {
  fn get(&self, name: &str) -> Result<String> {
    let name = format!("{}.{}", self.namespace, name);
    self.config.get_string(&name).chain_err(|| "")
  }

  fn set(&mut self, name: &str, value: &str) -> Result<()> {
    let name = format!("{}.{}", self.namespace, name);
    self.config.set_str(&name, value).chain_err(|| "")
  }
}
