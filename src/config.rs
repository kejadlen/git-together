use std::process::{Command, Output};
use errors::*;

pub trait Config {
  fn get(&self, name: &str) -> Result<String>;
  fn set(&self, name: &str, value: &str) -> Result<()>;
}

pub struct GitConfig {
  pub namespace: String,
}

impl GitConfig {
  fn output(&self, args: &[&str]) -> Result<Output> {
    Command::new("git")
      .arg("config")
      .args(args)
      .output()
      .chain_err(|| "failed to execute `git config`")
      .and_then(|output| {
        if output.status.success() {
          Ok(output)
        } else {
          Err(ErrorKind::GitConfig(output).into())
        }
      })
  }
}

impl Config for GitConfig {
  fn get(&self, name: &str) -> Result<String> {
    let name = format!("{}.{}", self.namespace, name);
    let output = try!(self.output(&[&name]));
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().into())
  }

  fn set(&self, name: &str, value: &str) -> Result<()> {
    let name = format!("{}.{}", self.namespace, name);
    self.output(&[&name, value]).and(Ok(()))
  }
}
