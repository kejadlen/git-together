#![recursion_limit = "1024"]
#[macro_use]

extern crate error_chain;

pub mod config;
mod errors;

use std::process::Command;

use config::Config;
use errors::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Author {
  pub name: String,
  pub email: String,
}

pub struct GitTogether<C> {
  pub config: C,
}

impl<C: Config> GitTogether<C> {
  pub fn set_active(&self, inits: &[&str]) -> Result<()> {
    self.get_authors(inits)
      .and_then(|_| self.config.set("active", &inits.join("+")))
  }

  pub fn signoff<'a>(&self,
                         cmd: &'a mut Command)
                         -> Result<&'a mut Command> {
    let active = try!(self.get_active());
    let mut active_iter = active.iter();

    let cmd = match active_iter.next() {
      Some(author) => {
        cmd.env("GIT_AUTHOR_NAME", author.name.clone())
          .env("GIT_AUTHOR_EMAIL", author.email.clone())
      }
      _ => cmd,
    };

    let cmd = match active_iter.next() {
      Some(committer) => {
        cmd.env("GIT_COMMITTER_NAME", committer.name.clone())
          .env("GIT_COMMITTER_EMAIL", committer.email.clone())
          .arg("--signoff")
      }
      _ => cmd,
    };

    Ok(cmd)
  }

  fn get_active(&self) -> Result<Vec<Author>> {
    self.config.get("active").and_then(|inits| {
      let inits: Vec<_> = inits.split('+').collect();
      self.get_authors(&inits)
    })
  }

  fn get_authors(&self, inits: &[&str]) -> Result<Vec<Author>> {
    let domain = try!(self.config.get("domain"));
    inits.iter()
      .map(|&init| {
        self.config
          .get(&format!("authors.{}", init))
          .chain_err(|| ErrorKind::AuthorNotFound(init.into()))
          .and_then(|raw| {
            if raw.is_empty() {
              return Err(ErrorKind::InvalidAuthor(raw).into());
            }

            let mut split = raw.split(';');
            let name = try!(split.next()
                .ok_or(ErrorKind::InvalidAuthor(raw.clone())))
              .trim();
            let username = try!(split.next()
                .ok_or(ErrorKind::InvalidAuthor(raw.clone())))
              .trim();
            let email = format!("{}@{}", username, domain);

            Ok(Author {
              name: name.into(),
              email: email,
            })
          })
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::cell::RefCell;
  use std::collections::HashMap;

  use config::Config;
  use errors::*;

  #[test]
  fn get_authors_no_domain() {
    let config = MockConfig::new(&[("authors.jh", "James Holden; jholden")]);
    let gt = GitTogether { config: config };

    assert!(gt.get_authors(&["jh"]).is_err());
  }

  #[test]
  fn get_authors() {
    let config = MockConfig::new(&[("domain", "rocinante.com"),
                                   ("authors.jh", ""),
                                   ("authors.nn", "Naomi Nagata"),
                                   ("authors.ab", "Amos Burton; aburton"),
                                   ("authors.ak", "Alex Kamal; akamal") /* ("authors.ca", "Chrisjen Avasarala;"),
                                                                         * ("authors.bd", "Bobbie Draper; bdraper@mars.mil"),
                                                                         * ("authors.jm", "Joe Miller; jmiller@starhelix.com"), */]);
    let gt = GitTogether { config: config };

    assert!(gt.get_authors(&["jh"]).is_err());
    assert!(gt.get_authors(&["nn"]).is_err());
    // assert!(gt.get_authors(&["ca"]).is_err());
    // assert!(gt.get_authors(&["jh", "bd"]).is_err());

    assert_eq!(gt.get_authors(&["ab", "ak"]).unwrap(),
               vec![
               Author { name: "Amos Burton".into(), email: "aburton@rocinante.com".into() },
               Author { name: "Alex Kamal".into(), email: "akamal@rocinante.com".into() },
               ]);
    // assert_eq!(gt.get_authors(&["ab", "bd", "jm"]).unwrap(),
    //            vec![Author { name: "Amos Burton".into(), email: "aburton@rocinante.com".into() },
    // Author { name: "Bobbie Draper".into(), email: "bdraper@mars.mil".into() },
    // Author { name: "Joe Miller".into(), email: "jmiller@starhelix.com".into() },
    // ]);
  }

  #[test]
  fn set_active() {
    let config = MockConfig::new(&[("domain", "rocinante.com"),
                                   ("authors.jh", "James Holden; jholden"),
                                   ("authors.nn", "Naomi Nagata; nnagata")]);
    let gt = GitTogether { config: config };

    gt.set_active(&["jh"]).unwrap();
    assert_eq!(gt.config.get("active").unwrap(), "jh".to_string());

    gt.set_active(&["jh", "nn"]).unwrap();
    assert_eq!(gt.config.get("active").unwrap(), "jh+nn".to_string());
  }

  struct MockConfig {
    data: RefCell<HashMap<String, String>>,
  }

  impl MockConfig {
    fn new(data: &[(&str, &str)]) -> MockConfig {
      let data = data.iter()
        .map(|&(k, v)| (k.into(), v.into()))
        .collect();
      MockConfig { data: RefCell::new(data) }
    }
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
