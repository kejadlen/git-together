use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Author {
  pub name: String,
  pub email: String,
}

pub struct AuthorFactory {
  pub domain: String,
}

impl AuthorFactory {
  // NOTE This doesn't check domain at all.
  pub fn parse(&self, raw: &str) -> Option<Author> {
    let mut split = raw.split(';').map(str::trim);

    let name = match split.next() {
      Some(name) if !name.is_empty() => name,
      _ => { return None; },
    };

    let email_seed = match split.next() {
      Some(email_seed) if !email_seed.is_empty() => email_seed,
      _ => { return None; },
    };

    let email = if email_seed.contains('@') {
      email_seed.into()
    } else {
      format!("{}@{}", email_seed, self.domain)
    };

    Some(Author {
      name: name.into(),
      email: email,
    })
  }
}

impl fmt::Display for Author {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} <{}>", self.name, self.email)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let author_factory = AuthorFactory { domain: "example.com".into() };

    let author = author_factory.parse("Jane Doe; jdoe").unwrap();
    assert_eq!(author.name, "Jane Doe");
    assert_eq!(author.email, "jdoe@example.com");

    let author = author_factory.parse("");
    assert!(author.is_none());

    let author = author_factory.parse("Jane Doe");
    assert!(author.is_none());

    let author = author_factory.parse("Jane Doe; ");
    assert!(author.is_none());

    let author = author_factory.parse("Jane Doe; jane.doe@example.edu").unwrap();
    assert_eq!(author.name, "Jane Doe");
    assert_eq!(author.email, "jane.doe@example.edu");
  }
}
