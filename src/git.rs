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

  pub fn auto_include(&mut self) {
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

    if self.already_included(&include_path).unwrap_or(true) {
      return;
    }

    let _ = Command::new("git")
      .args(&["config", "--add", "include.path", &include_path])
      .status();
  }

  fn already_included(&self, include_path: &str) -> Result<bool> {
    let local_config =
      try!(self.config.open_level(git2::ConfigLevel::Local).chain_err(|| ""));
    let entries = try!(local_config.entries(None).chain_err(|| ""));
    Ok(IntoIterator::into_iter(&entries).any(|entry| {
      entry.map(|entry| entry.value() == Some(include_path)).unwrap_or(true)
    }))
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
