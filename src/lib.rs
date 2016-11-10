#![feature(advanced_slice_patterns, slice_patterns)]
#![recursion_limit = "1024"]
#[macro_use]

extern crate error_chain;
extern crate git2;

pub mod errors;
pub mod git;

use std::collections::HashMap;
use std::fmt;
use std::process::Command;

use errors::*;
use git::Config;

#[derive(Clone, Debug, PartialEq)]
pub struct Author {
  pub name: String,
  pub email: String,
}

impl fmt::Display for Author {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} <{}>", self.name, self.email)
  }
}

pub struct GitTogether<C> {
  pub config: C,
}

impl<C: Config> GitTogether<C> {
  pub fn set_active(&mut self, inits: &[&str]) -> Result<Vec<Author>> {
    let authors = try!(self.get_authors(inits));
    try!(self.config.set("active", &inits.join("+")));
    Ok(authors)
  }

  pub fn all_authors(&self) -> Result<HashMap<String, Author>> {
    let mut authors = HashMap::new();
    let domain = try!(self.config.get("domain"));
    let raw = try!(self.config.get_all("authors."));
    for (name, value) in raw {
      let initials = try!(name.split('.').last().ok_or(""));
      let author = try!(Self::author(&domain, &value));
      authors.insert(initials.into(), author);
    }
    Ok(authors)
  }

  pub fn signoff<'a>(&self, cmd: &'a mut Command) -> Result<&'a mut Command> {
    let active = try!(self.config.get("active"));
    let inits: Vec<_> = active.split('+').collect();
    let authors = try!(self.get_authors(&inits));

    let (author, committer) = match authors.as_slice() {
      &[] => {
        return Err("".into());
      }
      &[ref solo] => (solo, solo),
      &[ref author, ref committer, _..] => (author, committer),
    };

    let cmd = cmd.env("GIT_AUTHOR_NAME", author.name.clone())
      .env("GIT_AUTHOR_EMAIL", author.email.clone())
      .env("GIT_COMMITTER_NAME", committer.name.clone())
      .env("GIT_COMMITTER_EMAIL", committer.email.clone());

    let cmd = if author != committer {
      cmd.arg("--signoff")
    } else {
      cmd
    };

    Ok(cmd)
  }

  fn get_active(&self) -> Result<Vec<String>> {
    self.config
      .get("active")
      .map(|active| active.split('+').map(|s| s.into()).collect())
  }

  pub fn rotate_active(&mut self) -> Result<()> {
    self.get_active().and_then(|active| {
      let mut inits: Vec<_> = active.iter().map(String::as_ref).collect();
      if !inits.is_empty() {
        let author = inits.remove(0);
        inits.push(author);
      }
      self.set_active(&inits[..]).map(|_| ())
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

            Self::author(&domain, &raw)
          })
      })
      .collect()
  }

  fn author(domain: &str, raw: &str) -> Result<Author> {
    let split: Vec<_> = raw.split(';').collect();
    if split.len() < 2 {
      return Err(ErrorKind::InvalidAuthor(raw.into()).into());
    }

    let name = split[0].trim().to_string();
    if name.is_empty() {
      return Err(ErrorKind::InvalidAuthor(raw.into()).into());
    }

    let email_seed = split[1].trim();
    if email_seed.is_empty() {
      return Err(ErrorKind::InvalidAuthor(raw.into()).into());
    }

    let email = if email_seed.contains('@') {
      email_seed.into()
    } else {
      format!("{}@{}", email_seed, domain)
    };

    Ok(Author {
      name: name,
      email: email,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::collections::HashMap;

  use errors::*;
  use git::Config;

  #[test]
  fn get_authors_no_domain() {
    let config = MockConfig::new(&[("authors.jh", "James Holden; jholden")]);
    let gt = GitTogether { config: config };

    assert!(gt.get_authors(&["jh"]).is_err());
  }

  #[test]
  fn get_authors() {
    let config =
      MockConfig::new(&[("domain", "rocinante.com"),
                        ("authors.jh", ""),
                        ("authors.nn", "Naomi Nagata"),
                        ("authors.ab", "Amos Burton; aburton"),
                        ("authors.ak", "Alex Kamal; akamal"),
                        ("authors.ca", "Chrisjen Avasarala;"),
                        ("authors.bd", "Bobbie Draper; bdraper@mars.mil"),
                        ("authors.jm", "Joe Miller; jmiller@starhelix.com")]);
    let gt = GitTogether { config: config };

    assert!(gt.get_authors(&["jh"]).is_err());
    assert!(gt.get_authors(&["nn"]).is_err());
    assert!(gt.get_authors(&["ca"]).is_err());
    assert!(gt.get_authors(&["jh", "bd"]).is_err());

    assert_eq!(gt.get_authors(&["ab", "ak"]).unwrap(),
               vec![Author {
                      name: "Amos Burton".into(),
                      email: "aburton@rocinante.com".into(),
                    },
                    Author {
                      name: "Alex Kamal".into(),
                      email: "akamal@rocinante.com".into(),
                    }]);
    assert_eq!(gt.get_authors(&["ab", "bd", "jm"]).unwrap(),
               vec![Author {
                      name: "Amos Burton".into(),
                      email: "aburton@rocinante.com".into(),
                    },
                    Author {
                      name: "Bobbie Draper".into(),
                      email: "bdraper@mars.mil".into(),
                    },
                    Author {
                      name: "Joe Miller".into(),
                      email: "jmiller@starhelix.com".into(),
                    }]);
  }

  #[test]
  fn set_active() {
    let config = MockConfig::new(&[("domain", "rocinante.com"),
                                   ("authors.jh", "James Holden; jholden"),
                                   ("authors.nn", "Naomi Nagata; nnagata")]);
    let mut gt = GitTogether { config: config };

    gt.set_active(&["jh"]).unwrap();
    assert_eq!(gt.get_active().unwrap(), vec!["jh"]);

    gt.set_active(&["jh", "nn"]).unwrap();
    assert_eq!(gt.get_active().unwrap(), vec!["jh", "nn"]);
  }

  #[test]
  fn rotate_active() {
    let config = MockConfig::new(&[("active", "jh+nn"),
                                   ("domain", "rocinante.com"),
                                   ("authors.jh", "James Holden; jholden"),
                                   ("authors.nn", "Naomi Nagata; nnagata")]);
    let mut gt = GitTogether { config: config };

    gt.rotate_active().unwrap();
    assert_eq!(gt.get_active().unwrap(), vec!["nn", "jh"]);
  }

  #[test]
  fn all_authors() {
    let config =
      MockConfig::new(&[("active", "jh+nn"),
                        ("domain", "rocinante.com"),
                        ("authors.ab", "Amos Burton; aburton"),
                        ("authors.bd", "Bobbie Draper; bdraper@mars.mil"),
                        ("authors.jm", "Joe Miller; jmiller@starhelix.com")]);
    let gt = GitTogether { config: config };

    let all_authors = gt.all_authors().unwrap();
    assert_eq!(all_authors.len(), 3);
    assert_eq!(all_authors["ab"],
               Author {
                 name: "Amos Burton".into(),
                 email: "aburton@rocinante.com".into(),
               });
    assert_eq!(all_authors["bd"],
               Author {
                 name: "Bobbie Draper".into(),
                 email: "bdraper@mars.mil".into(),
               });
    assert_eq!(all_authors["jm"],
               Author {
                 name: "Joe Miller".into(),
                 email: "jmiller@starhelix.com".into(),
               });
  }

  struct MockConfig {
    data: HashMap<String, String>,
  }

  impl MockConfig {
    fn new(data: &[(&str, &str)]) -> MockConfig {
      MockConfig {
        data: data.iter().map(|&(k, v)| (k.into(), v.into())).collect(),
      }
    }
  }

  impl Config for MockConfig {
    fn get(&self, name: &str) -> Result<String> {
      self.data
        .get(name.into())
        .cloned()
        .ok_or(format!("name not found: '{}'", name).into())
    }

    fn get_all(&self, glob: &str) -> Result<HashMap<String, String>> {
      Ok(self.data
        .iter()
        .filter(|&(name, _)| name.contains(glob))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect())
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
      self.data.insert(name.into(), value.into());
      Ok(())
    }
  }
}
