#![recursion_limit = "1024"]
#[macro_use]

extern crate error_chain;

pub mod config;
mod errors;

use config::Config;
use std::process::Command;

pub struct GitTogether<C: Config> {
  pub config: C,
}

impl<C: Config> GitTogether<C> {
  pub fn set_authors(&mut self, inits: &[&str]) {
    let domain = self.config.get("domain").unwrap();
    for init in inits {
      let raw = self.config.get(&format!("authors.{}", init)).unwrap();
      let mut split = raw.split(';');
      let name = split.next().unwrap().trim();
      let local_part = split.next().unwrap().trim();
      let email = format!("{}@{}", local_part, domain);

      self.config.set("author-name", name).unwrap();
      self.config.set("author-email", &email).unwrap();
    }
  }

  pub fn signoff<'a>(&self, cmd: &'a mut Command) -> &'a mut Command {
    let author_name = self.config.get("author-name").unwrap();
    let author_email = self.config.get("author-email").unwrap();
    cmd.env("GIT_AUTHOR_NAME", author_name)
      .env("GIT_AUTHOR_EMAIL", author_email)
      .arg("--signoff")
  }
}

#[derive(Clone)]
pub struct Author {}

#[cfg(test)]
mod tests {
  use super::*;

  use std::collections::HashMap;

  use config::Config;
  use errors::*;

  #[test]
  fn set_authors() {
    let data = vec![
      ("domain", "rocinante.com"),
      ("authors.jh", "James Holden; jholden"),
      ("authors.nn", "Naomi Nagata; nnagata"),
    ]
      .iter()
      .map(|&(k, v)| (k.into(), v.into()))
      .collect();
    let config = MockConfig { data: data };
    let mut gt = GitTogether { config: config };

    gt.set_authors(&["jh"]);

    assert_eq!(gt.config.get("author-name").unwrap(),
               "James Holden".to_string());
    assert_eq!(gt.config.get("author-email").unwrap(),
               "jholden@rocinante.com".to_string());
  }

  struct MockConfig {
    data: HashMap<String, String>,
  }

  impl Config for MockConfig {
    fn get(&self, name: &str) -> Result<String> {
      self.data.get(name.into()).cloned().ok_or("".into())
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
      self.data.insert(name.into(), value.into());
      Ok(())
    }
  }
}
