use std::path::PathBuf;
use std::process::{Command, Output};
use errors::*;

pub trait Config {
  fn get(&self, name: &str) -> Result<String>;
  fn set(&self, name: &str, value: &str) -> Result<()>;
}

pub fn root() -> Result<PathBuf> {
  let output =
    try!(Command::new("git").args(&["rev-parse", "--show-toplevel"]).output());
  let stdout = String::from_utf8_lossy(&output.stdout);

  Ok(stdout.trim_right().into())
}

pub struct GitConfig {
  pub namespace: String,
}

impl GitConfig {
  pub fn auto_include(&self) {
    // Make sure .git-together exists
    if let Ok(mut path) = root() {
      path.push(&format!(".{}", self.namespace));
      if !path.exists() {
        return;
      }
    } else {
      return;
    }

    // Make sure we're not already including .git-together
    if let Ok(output) = self.output(&["--local", "--get-all", "include.path"]) {
      let stdout = String::from_utf8_lossy(&output.stdout);
      if stdout.split('\n').any(|x| x == "../git-together") {
        return;
      }
    }

    let _ = self.output(&["--add", "include.path", &format!("../.{}", self.namespace)]);
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
    let output = try!(self.output(&[&name]));
    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(stdout.trim_right().into())
  }

  fn set(&self, name: &str, value: &str) -> Result<()> {
    let name = format!("{}.{}", self.namespace, name);

    self.output(&[&name, value]).and(Ok(()))
  }
}
