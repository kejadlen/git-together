use std::fmt;

use errors::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Author {
    pub name: String,
    pub email: String,
}

impl Author {
    pub fn new(raw: &str, domain: Option<&str>) -> Result<Author> {
        let mut split = raw.split(';').map(str::trim);

        let name = match split.next() {
            Some(name) if !name.is_empty() => name,
            _ => {
                return Err("missing name".into());
            }
        };

        let email_seed = match split.next() {
            Some(email_seed) if !email_seed.is_empty() => email_seed,
            _ => {
                return Err("missing email seed".into());
            }
        };

        let email = if email_seed.contains('@') {
            email_seed.into()
        } else if let Some(domain) = domain {
            format!("{}@{}", email_seed, domain)
        } else {
            return Err("missing domain".into());
        };

        Ok(Author {
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

#[test]
fn test_new_author() {
    let author = Author::new("Jane Doe; jdoe", Some("example.com")).unwrap();
    assert_eq!(author.name, "Jane Doe");
    assert_eq!(author.email, "jdoe@example.com");

    let author = Author::new("", Some("example.com"));
    assert!(author.is_err());

    let author = Author::new("Jane Doe", Some("example.com"));
    assert!(author.is_err());

    let author = Author::new("Jane Doe;", Some("example.com"));
    assert!(author.is_err());

    let author = Author::new("Jane Doe; jane.doe@example.edu", Some("example.com")).unwrap();
    assert_eq!(author.name, "Jane Doe");
    assert_eq!(author.email, "jane.doe@example.edu");

    let author = Author::new("Jane Doe; jane.doe@example.edu", None).unwrap();
    assert_eq!(author.name, "Jane Doe");
    assert_eq!(author.email, "jane.doe@example.edu");

    let author = Author::new("Jane Doe; jane.doe", None);
    assert!(author.is_err());
}
