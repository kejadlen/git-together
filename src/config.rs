use std::process::Command;
use errors::*;

pub trait Config {
  fn get(&self, name: &str) -> Result<String>;
  fn set(&mut self, name: &str, value: &str) -> Result<()>;
}

pub struct GitConfig {
  pub namespace: String,
}

impl Config for GitConfig {
  fn get(&self, name: &str) -> Result<String> {
    let name = format!("{}.{}", self.namespace, name);
    let output = try!(Command::new("git")
      .args(&["config", &name])
      .output()
      .chain_err(|| "failed to execute `git config`"));

    if !output.status.success() {
      let err = format!("failed to execute `git config`: {}",
                        String::from_utf8_lossy(&output.stderr).trim());
      return Err(err.into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().into())
  }

  fn set(&mut self, name: &str, value: &str) -> Result<()> {
    let name = format!("{}.{}", self.namespace, name);
    try!(Command::new("git")
      .args(&["config", &name, &value])
      .status()
      .chain_err(|| "failed to execute `git config`"));
    Ok(())
  }
}
