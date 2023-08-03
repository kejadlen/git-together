#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate git2;

pub mod author;
pub mod config;
pub mod errors;
pub mod git;

use std::collections::HashMap;
use std::env;
use std::process::Command;

use author::{Author, AuthorParser};
use config::Config;
use errors::*;

const NAMESPACE: &str = "git-together";
const TRIGGERS: [&str; 2] = ["with", "together"];

fn namespaced(name: &str) -> String {
    format!("{}.{}", NAMESPACE, name)
}

pub fn run() -> Result<i32> {
    let all_args: Vec<_> = env::args().skip(1).collect();
    let mut args: Vec<&str> = all_args.iter().map(String::as_ref).collect();

    let mut gt = if args.contains(&"--global") {
        GitTogether::new(ConfigScope::Global)
    } else {
        GitTogether::new(ConfigScope::Local)
    }?;

    args.retain(|&arg| arg != "--global");

    let mut skip_next = false;
    let command = args
        .iter()
        .find(|x| {
            if skip_next {
                skip_next = false;
                return false;
            }
            match x {
                &&"-c" | &&"--exec-path" | &&"--git-dir" | &&"--work-tree" | &&"--namespace"
                | &&"--super-prefix" | &&"--list-cmds" | &&"-C" => {
                    skip_next = true;
                    false
                }
                &&"--version" | &&"--help" => true,
                v if v.starts_with('-') => false,
                _ => true,
            }
        })
        .unwrap_or(&"");

    let mut split_args = args.split(|x| x == command);
    let global_args = split_args.next().unwrap_or(&[]);
    let command_args = split_args.next().unwrap_or(&[]);

    let code = if TRIGGERS.contains(command) {
        match command_args {
            [] => {
                let inits = gt.get_active()?;
                let inits: Vec<_> = inits.iter().map(String::as_ref).collect();
                let authors = gt.get_authors(&inits)?;

                for (initials, author) in inits.iter().zip(authors.iter()) {
                    println!("{}: {}", initials, author);
                }
            }
            ["--list"] => {
                let authors = gt.all_authors()?;
                let mut sorted: Vec<_> = authors.iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(b.0));

                for (initials, author) in sorted {
                    println!("{}: {}", initials, author);
                }
            }
            ["--clear"] => {
                gt.clear_active()?;
            }
            ["--version"] => {
                println!(
                    "{} {}",
                    option_env!("CARGO_PKG_NAME").unwrap_or("git-together"),
                    option_env!("CARGO_PKG_VERSION").unwrap_or("unknown version")
                );
            }
            _ => {
                let authors = gt.set_active(command_args)?;
                for author in authors {
                    println!("{}", author);
                }
            }
        }

        0
    } else if gt.is_signoff_cmd(command) {
        if command == &"merge" {
            env::set_var("GIT_TOGETHER_NO_SIGNOFF", "1");
        }

        let mut cmd = Command::new("git");
        let cmd = cmd.args(global_args);
        let cmd = cmd.arg(command);
        let cmd = gt.signoff(cmd)?;
        let cmd = cmd.args(command_args);

        let status = cmd.status().chain_err(|| "failed to execute process")?;
        if status.success() {
            gt.rotate_active()?;
        }
        status.code().ok_or("process terminated by signal")?
    } else {
        let status = Command::new("git")
            .args(args)
            .status()
            .chain_err(|| "failed to execute process")?;
        status.code().ok_or("process terminated by signal")?
    };

    Ok(code)
}

pub struct GitTogether<C> {
    config: C,
    author_parser: AuthorParser,
}

pub enum ConfigScope {
    Local,
    Global,
}

impl GitTogether<git::Config> {
    pub fn new(scope: ConfigScope) -> Result<Self> {
        let config = match scope {
            ConfigScope::Local => {
                let repo = git::Repo::new();
                if let Ok(ref repo) = repo {
                    let _ = repo.auto_include(&format!(".{}", NAMESPACE));
                };

                repo.and_then(|r| r.config())
                    .or_else(|_| git::Config::new(scope))?
            }
            ConfigScope::Global => git::Config::new(scope)?,
        };

        let domain = config.get(&namespaced("domain")).ok();
        let author_parser = AuthorParser { domain };

        Ok(GitTogether {
            config,
            author_parser,
        })
    }
}

impl<C: config::Config> GitTogether<C> {
    pub fn set_active(&mut self, inits: &[&str]) -> Result<Vec<Author>> {
        let authors = self.get_authors(inits)?;
        self.config.set(&namespaced("active"), &inits.join("+"))?;

        self.save_original_user()?;
        if let Some(author) = authors.get(0) {
            self.set_user(&author.name, &author.email)?;
        }

        Ok(authors)
    }

    pub fn clear_active(&mut self) -> Result<()> {
        self.config.clear(&namespaced("active"))?;

        let _ = self.config.clear("user.name");
        let _ = self.config.clear("user.email");

        Ok(())
    }

    fn save_original_user(&mut self) -> Result<()> {
        if let Ok(name) = self.config.get("user.name") {
            let key = namespaced("user.name");
            self.config
                .get(&key)
                .map(|_| ())
                .or_else(|_| self.config.set(&key, &name))?;
        }

        if let Ok(email) = self.config.get("user.email") {
            let key = namespaced("user.email");
            self.config
                .get(&key)
                .map(|_| ())
                .or_else(|_| self.config.set(&key, &email))?;
        }

        Ok(())
    }

    fn set_user(&mut self, name: &str, email: &str) -> Result<()> {
        self.config.set("user.name", name)?;
        self.config.set("user.email", email)?;

        Ok(())
    }

    pub fn all_authors(&self) -> Result<HashMap<String, Author>> {
        let mut authors = HashMap::new();
        let raw = self.config.get_all(&namespaced("authors."))?;
        for (name, value) in raw {
            let initials = name.split('.').last().ok_or("")?;
            let author = self.parse_author(initials, &value)?;
            authors.insert(initials.into(), author);
        }
        Ok(authors)
    }

    pub fn is_signoff_cmd(&self, cmd: &str) -> bool {
        let signoffs = ["commit", "merge", "revert"];
        signoffs.contains(&cmd) || self.is_signoff_alias(cmd)
    }

    fn is_signoff_alias(&self, cmd: &str) -> bool {
        self.config
            .get(&namespaced("aliases"))
            .unwrap_or_else(|_| String::new())
            .split(',')
            .filter(|s| s != &"")
            .any(|a| a == cmd)
    }

    pub fn signoff<'a>(&self, cmd: &'a mut Command) -> Result<&'a mut Command> {
        let active = self.config.get(&namespaced("active"))?;
        let inits: Vec<_> = active.split('+').collect();
        let authors = self.get_authors(&inits)?;

        let (author, committer) = match *authors.as_slice() {
            [] => {
                return Err("".into());
            }
            [ref solo] => (solo, solo),
            [ref author, ref committer, ..] => (author, committer),
        };

        let cmd = cmd
            .env("GIT_AUTHOR_NAME", author.name.clone())
            .env("GIT_AUTHOR_EMAIL", author.email.clone())
            .env("GIT_COMMITTER_NAME", committer.name.clone())
            .env("GIT_COMMITTER_EMAIL", committer.email.clone());

        let no_signoff = env::var("GIT_TOGETHER_NO_SIGNOFF").is_ok();
        Ok(if !no_signoff && author != committer {
            cmd.arg("--signoff")
        } else {
            cmd
        })
    }

    fn get_active(&self) -> Result<Vec<String>> {
        self.config
            .get(&namespaced("active"))
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
        inits
            .iter()
            .map(|&initials| self.get_author(initials))
            .collect()
    }

    fn get_author(&self, initials: &str) -> Result<Author> {
        self.config
            .get(&namespaced(&format!("authors.{}", initials)))
            .chain_err(|| format!("author not found for '{}'", initials))
            .and_then(|raw| self.parse_author(initials, &raw))
    }

    fn parse_author(&self, initials: &str, raw: &str) -> Result<Author> {
        self.author_parser
            .parse(raw)
            .chain_err(|| format!("invalid author for '{}': '{}'", initials, raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::ops::Index;

    use author::{Author, AuthorParser};
    use config::Config;

    #[test]
    fn get_authors() {
        let config = MockConfig::new(&[
            ("git-together.authors.jh", ""),
            ("git-together.authors.nn", "Naomi Nagata"),
            ("git-together.authors.ab", "Amos Burton; aburton"),
            ("git-together.authors.ak", "Alex Kamal; akamal"),
            ("git-together.authors.ca", "Chrisjen Avasarala;"),
            ("git-together.authors.bd", "Bobbie Draper; bdraper@mars.mil"),
            (
                "git-together.authors.jm",
                "Joe Miller; jmiller@starhelix.com",
            ),
        ]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let gt = GitTogether {
            config,
            author_parser,
        };

        assert!(gt.get_authors(&["jh"]).is_err());
        assert!(gt.get_authors(&["nn"]).is_err());
        assert!(gt.get_authors(&["ca"]).is_err());
        assert!(gt.get_authors(&["jh", "bd"]).is_err());

        assert_eq!(
            gt.get_authors(&["ab", "ak"]).unwrap(),
            vec![
                Author {
                    name: "Amos Burton".into(),
                    email: "aburton@rocinante.com".into(),
                },
                Author {
                    name: "Alex Kamal".into(),
                    email: "akamal@rocinante.com".into(),
                }
            ]
        );
        assert_eq!(
            gt.get_authors(&["ab", "bd", "jm"]).unwrap(),
            vec![
                Author {
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
                }
            ]
        );
    }

    #[test]
    fn set_active_solo() {
        let config = MockConfig::new(&[
            ("git-together.authors.jh", "James Holden; jholden"),
            ("git-together.authors.nn", "Naomi Nagata; nnagata"),
            ("user.name", "Bobbie Draper"),
            ("user.email", "bdraper@mars.mil"),
        ]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let mut gt = GitTogether {
            config,
            author_parser,
        };

        gt.set_active(&["jh"]).unwrap();
        assert_eq!(gt.get_active().unwrap(), vec!["jh"]);
        assert_eq!(gt.config["user.name"], "James Holden");
        assert_eq!(gt.config["user.email"], "jholden@rocinante.com");
        assert_eq!(gt.config["git-together.user.name"], "Bobbie Draper");
        assert_eq!(gt.config["git-together.user.email"], "bdraper@mars.mil");
    }

    #[test]
    fn set_active_pair() {
        let config = MockConfig::new(&[
            ("git-together.authors.jh", "James Holden; jholden"),
            ("git-together.authors.nn", "Naomi Nagata; nnagata"),
            ("user.name", "Bobbie Draper"),
            ("user.email", "bdraper@mars.mil"),
        ]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let mut gt = GitTogether {
            config,
            author_parser,
        };

        gt.set_active(&["nn", "jh"]).unwrap();
        assert_eq!(gt.get_active().unwrap(), vec!["nn", "jh"]);
        assert_eq!(gt.config["user.name"], "Naomi Nagata");
        assert_eq!(gt.config["user.email"], "nnagata@rocinante.com");
        assert_eq!(gt.config["git-together.user.name"], "Bobbie Draper");
        assert_eq!(gt.config["git-together.user.email"], "bdraper@mars.mil");
    }

    #[test]
    fn clear_active_pair() {
        let config = MockConfig::new(&[
            ("git-together.authors.jh", "James Holden; jholden"),
            ("git-together.authors.nn", "Naomi Nagata; nnagata"),
            ("user.name", "Bobbie Draper"),
            ("user.email", "bdraper@mars.mil"),
        ]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let mut gt = GitTogether {
            config,
            author_parser,
        };

        gt.set_active(&["nn", "jh"]).unwrap();
        gt.clear_active().unwrap();
        assert!(gt.get_active().is_err());
        assert!(gt.config.get("user.name").is_err());
        assert!(gt.config.get("user.email").is_err());
    }

    #[test]
    fn multiple_set_active() {
        let config = MockConfig::new(&[
            ("git-together.authors.jh", "James Holden; jholden"),
            ("git-together.authors.nn", "Naomi Nagata; nnagata"),
            ("user.name", "Bobbie Draper"),
            ("user.email", "bdraper@mars.mil"),
        ]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let mut gt = GitTogether {
            config,
            author_parser,
        };

        gt.set_active(&["nn"]).unwrap();
        gt.set_active(&["jh"]).unwrap();
        assert_eq!(gt.config["git-together.user.name"], "Bobbie Draper");
        assert_eq!(gt.config["git-together.user.email"], "bdraper@mars.mil");
    }

    #[test]
    fn rotate_active() {
        let config = MockConfig::new(&[
            ("git-together.active", "jh+nn"),
            ("git-together.authors.jh", "James Holden; jholden"),
            ("git-together.authors.nn", "Naomi Nagata; nnagata"),
        ]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let mut gt = GitTogether {
            config,
            author_parser,
        };

        gt.rotate_active().unwrap();
        assert_eq!(gt.get_active().unwrap(), vec!["nn", "jh"]);
    }

    #[test]
    fn all_authors() {
        let config = MockConfig::new(&[
            ("git-together.active", "jh+nn"),
            ("git-together.authors.ab", "Amos Burton; aburton"),
            ("git-together.authors.bd", "Bobbie Draper; bdraper@mars.mil"),
            (
                "git-together.authors.jm",
                "Joe Miller; jmiller@starhelix.com",
            ),
        ]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let gt = GitTogether {
            config,
            author_parser,
        };

        let all_authors = gt.all_authors().unwrap();
        assert_eq!(all_authors.len(), 3);
        assert_eq!(
            all_authors["ab"],
            Author {
                name: "Amos Burton".into(),
                email: "aburton@rocinante.com".into(),
            }
        );
        assert_eq!(
            all_authors["bd"],
            Author {
                name: "Bobbie Draper".into(),
                email: "bdraper@mars.mil".into(),
            }
        );
        assert_eq!(
            all_authors["jm"],
            Author {
                name: "Joe Miller".into(),
                email: "jmiller@starhelix.com".into(),
            }
        );
    }

    #[test]
    fn is_signoff_cmd_basics() {
        let config = MockConfig::new(&[]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let gt = GitTogether {
            config,
            author_parser,
        };

        assert_eq!(gt.is_signoff_cmd("commit"), true);
        assert_eq!(gt.is_signoff_cmd("merge"), true);
        assert_eq!(gt.is_signoff_cmd("revert"), true);
        assert_eq!(gt.is_signoff_cmd("bisect"), false);
    }

    #[test]
    fn is_signoff_cmd_aliases() {
        let config = MockConfig::new(&[("git-together.aliases", "ci,m,rv")]);
        let author_parser = AuthorParser {
            domain: Some("rocinante.com".into()),
        };
        let gt = GitTogether {
            config,
            author_parser,
        };

        assert_eq!(gt.is_signoff_cmd("ci"), true);
        assert_eq!(gt.is_signoff_cmd("m"), true);
        assert_eq!(gt.is_signoff_cmd("rv"), true);
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

    impl<'a> Index<&'a str> for MockConfig {
        type Output = String;

        fn index(&self, key: &'a str) -> &String {
            self.data.index(key)
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
            Ok(self
                .data
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

        fn clear(&mut self, name: &str) -> Result<()> {
            self.data.remove(name.into());
            Ok(())
        }
    }
}
