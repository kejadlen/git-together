#![recursion_limit = "1024"]
#[macro_use]

extern crate error_chain;

pub mod config;
mod errors;

use std::process::Command;

use config::Config;
use errors::*;

pub struct GitTogether<C> {
  pub config: C,
}

impl<C: Config> GitTogether<C> {
  pub fn set_authors(&mut self, inits: &[&str]) -> Result<()> {
    let domain = try!(self.config.get("domain").chain_err(|| "domain not set"));
    for init in inits {
      let raw = try!(self.config
        .get(&format!("authors.{}", init))
        .chain_err(|| format!("author not found for `{}`", init)));
      let mut split = raw.split(';');
      let name = try!(split.next().ok_or("".to_string())).trim();
      let username = try!(split.next().ok_or("".to_string())).trim();
      let email = format!("{}@{}", username, domain);

      try!(self.config.set("author-name", name));
      try!(self.config.set("author-email", &email));
    }

    Ok(())
  }

  pub fn add_signoff<'a>(&self,
                         cmd: &'a mut Command)
                         -> Result<&'a mut Command> {
    let author_name =
      try!(self.config.get("author-name").chain_err(|| "author name not set"));
    let author_email = try!(self.config
      .get("author-email")
      .chain_err(|| "author email not set"));
    Ok(cmd.env("GIT_AUTHOR_NAME", author_name)
      .env("GIT_AUTHOR_EMAIL", author_email)
      .arg("--signoff"))
  }
}

#[derive(Clone)]
pub struct Author {}

#[cfg(test)]
mod tests {
  use super::*;

  use std::cell::RefCell;
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
    let config = MockConfig { data: RefCell::new(data) };
    let mut gt = GitTogether { config: config };

    gt.set_authors(&["jh"]).unwrap();

    assert_eq!(gt.config.get("author-name").unwrap(),
               "James Holden".to_string());
    assert_eq!(gt.config.get("author-email").unwrap(),
               "jholden@rocinante.com".to_string());
  }

  struct MockConfig {
    data: RefCell<HashMap<String, String>>,
  }

  impl Config for MockConfig {
    fn get(&self, name: &str) -> Result<String> {
      self.data.borrow().get(name.into()).cloned().ok_or("".into())
    }

    fn set(&self, name: &str, value: &str) -> Result<()> {
      self.data.borrow_mut().insert(name.into(), value.into());
      Ok(())
    }
  }
}
