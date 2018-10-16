use std::fmt;

use errors::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Author {
    pub name: String,
    pub email: String,
}

pub struct AuthorParser {
    pub domain: Option<String>,
}

impl AuthorParser {
    pub fn parse(&self, raw: &str) -> Result<Author> {
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
        } else {
            let domain = match self.domain {
                Some(ref domain) => domain,
                None => {
                    return Err("missing domain".into());
                }
            };
            format!("{}@{}", email_seed, domain)
        };

        Ok(Author {
            name: name.into(),
            email,
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
        let author_parser = AuthorParser {
            domain: Some("example.com".into()),
        };

        let author = author_parser.parse("Jane Doe; jdoe").unwrap();
        assert_eq!(author.name, "Jane Doe");
        assert_eq!(author.email, "jdoe@example.com");

        let author = author_parser.parse("");
        assert!(author.is_err());

        let author = author_parser.parse("Jane Doe");
        assert!(author.is_err());

        let author = author_parser.parse("Jane Doe; ");
        assert!(author.is_err());

        let author = author_parser
            .parse("Jane Doe; jane.doe@example.edu")
            .unwrap();
        assert_eq!(author.name, "Jane Doe");
        assert_eq!(author.email, "jane.doe@example.edu");
    }
}
