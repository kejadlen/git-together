use std::process::Command;

pub trait Config {
  fn get(&self, name: &str) -> Option<String>;
  fn set(&mut self, name: &str, value: &str);
}

pub struct GitConfig {
  pub namespace: String,
}

impl Config for GitConfig {
  fn get(&self, name: &str) -> Option<String> {
    let name = format!("{}.{}", self.namespace, name);
    let output =
      Command::new("git").args(&["config", &name]).output().unwrap();

    if !output.status.success() {
      return None;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    Some(stdout.trim().into())
  }

  fn set(&mut self, name: &str, value: &str) {
    let name = format!("{}.{}", self.namespace, name);
    Command::new("git").args(&["config", &name, &value]).status().unwrap();
  }
}
