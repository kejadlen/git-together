pub mod config;
use config::Config;

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

      self.config.set("author-name", name);
      self.config.set("author-email", &email);
    }
  }
}

#[derive(Clone)]
pub struct Author {}

#[cfg(test)]
mod tests {
  use super::*;
  use config::Config;
  use std::collections::HashMap;

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

    assert_eq!(gt.config.get("author-name"),
               Some("James Holden".to_string()));
    assert_eq!(gt.config.get("author-email"),
               Some("jholden@rocinante.com".to_string()));
  }

  struct MockConfig {
    data: HashMap<String, String>,
  }

  impl Config for MockConfig {
    fn get(&self, name: &str) -> Option<String> {
      self.data.get(name.into()).cloned()
    }

    fn set(&mut self, name: &str, value: &str) {
      self.data.insert(name.into(), value.into());
    }
  }
}
