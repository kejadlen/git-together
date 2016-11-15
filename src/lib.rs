#![feature(advanced_slice_patterns, slice_patterns)]
#![recursion_limit = "1024"]
#[macro_use]

extern crate error_chain;
extern crate git2;

pub mod author;
pub mod config;
pub mod errors;

use std::collections::HashMap;
use std::env;
use std::process::Command;

use author::{Author, AuthorParser};
use config::{Config, GitConfig, NamespacedConfig};
use errors::*;

pub struct GitTogether<C> {
  config: C,
  author_parser: AuthorParser,
}

impl GitTogether<NamespacedConfig<GitConfig>> {
  pub fn new(namespace: &str) -> Result<Self> {
    let mut config = Self::open_config()?;
    Self::auto_include(&mut config, namespace);

    let config = GitConfig { config: config };
    let config = NamespacedConfig::new(namespace, config);
    let domain = config.get("domain")?;
    let author_parser = AuthorParser { domain: domain };

    Ok(GitTogether {
      config: config,
      author_parser: author_parser,
    })
  }

  fn open_config() -> Result<git2::Config> {
    let repo = Self::open_repo()?;
    repo.config().or_else(|_| git2::Config::open_default()).chain_err(|| "")
  }

  fn auto_include(config: &mut git2::Config, namespace: &str) {
    let filename = format!(".{}", namespace);
    let include_path = format!("../{}", filename);

    let repo = match Self::open_repo() {
      Ok(repo) => repo,
      Err(_) => { return; }
    };

    let file_exists = repo.workdir().map(|path| {
      let mut path_buf = path.to_path_buf();
      path_buf.push(&filename);
      path_buf.exists()
    });

    // Make sure .git-together exists
    if !file_exists.unwrap_or(false) {
      return;
    }

    if Self::already_included(config, &include_path).unwrap_or(true) {
      return;
    }

    let _ = config.set_multivar("include.path", "^$", &include_path);
  }

  fn already_included(config: &git2::Config, include_path: &str) -> Result<bool> {
    let local_config = config
      .open_level(git2::ConfigLevel::Local)
      .chain_err(|| "error opening local git config")?;
    let entries = local_config.entries(None)
      .chain_err(|| "error getting git config entries")?;
    Ok(entries.into_iter().any(|entry| {
      entry.map(|entry| entry.value() == Some(include_path)).unwrap_or(true)
    }))
  }

  fn open_repo() -> Result<git2::Repository> {
    let path = env::current_dir().chain_err(|| "")?;
    git2::Repository::discover(path).chain_err(|| "")
  }
}

impl<C: Config> GitTogether<C> {
  pub fn set_active(&mut self, inits: &[&str]) -> Result<Vec<Author>> {
    let authors = self.get_authors(inits)?;
    self.config.set("active", &inits.join("+"))?;
    Ok(authors)
  }

  pub fn all_authors(&self) -> Result<HashMap<String, Author>> {
    let mut authors = HashMap::new();
    let raw = self.config.get_all("authors.")?;
    for (name, value) in raw {
      let initials = name.split('.').last().ok_or("")?;
      let author = self.parse_author(initials, &value)?;
      authors.insert(initials.into(), author);
    }
    Ok(authors)
  }

  pub fn signoff<'a>(&self, cmd: &'a mut Command) -> Result<&'a mut Command> {
    let active = self.config.get("active")?;
    let inits: Vec<_> = active.split('+').collect();
    let authors = self.get_authors(&inits)?;

    let (author, committer) = match *authors.as_slice() {
      [] => {
        return Err("".into());
      }
      [ref solo] => (solo, solo),
      [ref author, ref committer, _..] => (author, committer),
    };

    let cmd = cmd.env("GIT_AUTHOR_NAME", author.name.clone())
      .env("GIT_AUTHOR_EMAIL", author.email.clone())
      .env("GIT_COMMITTER_NAME", committer.name.clone())
      .env("GIT_COMMITTER_EMAIL", committer.email.clone());

    let no_signoff = env::var("GIT_TOGETHER_NO_SIGNOFF").is_ok();
    let cmd = if !no_signoff && author != committer {
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
    inits.iter()
      .map(|&initials| self.get_author(initials))
      .collect()
  }

  fn get_author(&self, initials: &str) -> Result<Author> {
    self.config
      .get(&format!("authors.{}", initials))
      .chain_err(|| format!("author not found for '{}'", initials))
      .and_then(|raw| self.parse_author(initials, &raw))
  }

  fn parse_author(&self, initials: &str, raw: &str) -> Result<Author> {
    self.author_parser
      .parse(raw)
      .ok_or(format!("invalid author for '{}': '{}'", initials, raw).into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::collections::HashMap;

  use author::{Author, AuthorParser};
  use config::Config;
  use errors::*;

  #[test]
  fn get_authors() {
    let config =
      MockConfig::new(&[("authors.jh", ""),
                        ("authors.nn", "Naomi Nagata"),
                        ("authors.ab", "Amos Burton; aburton"),
                        ("authors.ak", "Alex Kamal; akamal"),
                        ("authors.ca", "Chrisjen Avasarala;"),
                        ("authors.bd", "Bobbie Draper; bdraper@mars.mil"),
                        ("authors.jm", "Joe Miller; jmiller@starhelix.com")]);
    let author_parser = AuthorParser { domain: "rocinante.com".into() };
    let gt = GitTogether {
      config: config,
      author_parser: author_parser,
    };

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
    let config = MockConfig::new(&[("authors.jh", "James Holden; jholden"),
                                   ("authors.nn", "Naomi Nagata; nnagata")]);
    let author_parser = AuthorParser { domain: "rocinante.com".into() };
    let mut gt = GitTogether {
      config: config,
      author_parser: author_parser,
    };

    gt.set_active(&["jh"]).unwrap();
    assert_eq!(gt.get_active().unwrap(), vec!["jh"]);

    gt.set_active(&["jh", "nn"]).unwrap();
    assert_eq!(gt.get_active().unwrap(), vec!["jh", "nn"]);
  }

  #[test]
  fn rotate_active() {
    let config = MockConfig::new(&[("active", "jh+nn"),
                                   ("authors.jh", "James Holden; jholden"),
                                   ("authors.nn", "Naomi Nagata; nnagata")]);
    let author_parser = AuthorParser { domain: "rocinante.com".into() };
    let mut gt = GitTogether {
      config: config,
      author_parser: author_parser,
    };

    gt.rotate_active().unwrap();
    assert_eq!(gt.get_active().unwrap(), vec!["nn", "jh"]);
  }

  #[test]
  fn all_authors() {
    let config =
      MockConfig::new(&[("active", "jh+nn"),
                        ("authors.ab", "Amos Burton; aburton"),
                        ("authors.bd", "Bobbie Draper; bdraper@mars.mil"),
                        ("authors.jm", "Joe Miller; jmiller@starhelix.com")]);
    let author_parser = AuthorParser { domain: "rocinante.com".into() };
    let gt = GitTogether {
      config: config,
      author_parser: author_parser,
    };

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

    fn add(&mut self, _: &str, _: &str) -> Result<()> {
      unimplemented!();
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
      self.data.insert(name.into(), value.into());
      Ok(())
    }
  }
}
