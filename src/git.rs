use std::collections::HashMap;
use std::env;

use git2;

use errors::*;

pub trait Reader {
    fn get(&self, name: &str) -> Result<String>;
    fn get_all(&self, glob: &str) -> Result<HashMap<String, String>>;
}

pub trait Writer {
    fn add(&mut self, name: &str, value: &str) -> Result<()>;
    fn set(&mut self, name: &str, value: &str) -> Result<()>;
}

pub struct Repo {
    repo: git2::Repository,
}

pub struct Config {
    config: git2::Config,
}

impl Repo {
    pub fn new() -> Result<Self> {
        let repo = env::current_dir().chain_err(|| "").and_then(
            |current_dir| {
                git2::Repository::discover(current_dir).chain_err(|| "")
            },
        )?;
        Ok(Repo { repo: repo })
    }

    pub fn config(&self) -> Result<Config> {
        self.repo
            .config()
            .map(|config| Config { config: config })
            .chain_err(|| "")
    }

    pub fn auto_include(&self, filename: &str) -> Result<()> {
        let include_path = format!("../{}", filename);

        let workdir = match self.repo.workdir() {
            Some(dir) => dir,
            _ => {
                return Ok(());
            }
        };

        let mut path_buf = workdir.to_path_buf();
        path_buf.push(filename);
        if !path_buf.exists() {
            return Ok(());
        }

        let include_paths = self.include_paths()?;
        if include_paths.contains(&include_path) {
            return Ok(());
        }

        let mut config = self.local_config()?;
        config
            .set_multivar("include.path", "^$", &include_path)
            .and(Ok(()))
            .chain_err(|| "")
    }

    fn include_paths(&self) -> Result<Vec<String>> {
        let config = self.local_config()?;
        let include_paths: Vec<String> = config
            .entries(Some("include.path"))
            .chain_err(|| "error reading config entries")?
            .into_iter()
            .map(|entry| {
                entry.chain_err(|| "").and_then(|entry| {
                    entry.value().map(String::from).ok_or_else(|| "".into())
                })
            })
            .collect::<Result<_>>()?;
        Ok(include_paths)
    }

    fn local_config(&self) -> Result<git2::Config> {
        let config = self.repo.config().chain_err(|| "")?;
        config.open_level(git2::ConfigLevel::Local).chain_err(|| "")
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        git2::Config::open_default()
            .map(|config| Config { config: config })
            .chain_err(|| "error opening default git config")
    }

    pub fn global(&mut self) -> Result<Self> {
        self.config
            .open_global()
            .map(|config| Config { config: config })
            .chain_err(|| "error opening global git config")
    }
}

impl Reader for Config {
    fn get(&self, name: &str) -> Result<String> {
        self.config.get_string(name).chain_err(|| {
            format!("error getting git config for '{}'", name)
        })
    }

    fn get_all(&self, glob: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        let entries = self.config.entries(Some(glob)).chain_err(
            || "error getting git config entries",
        )?;
        for entry in &entries {
            let entry = entry.chain_err(|| "error getting git config entry")?;
            if let (Some(name), Some(value)) = (entry.name(), entry.value()) {
                result.insert(name.into(), value.into());
            }
        }
        Ok(result)
    }
}

impl Writer for Config {
    fn add(&mut self, name: &str, value: &str) -> Result<()> {
        self.config.set_multivar(name, "^$", value).chain_err(|| {
            format!("error adding git config '{}': '{}'", name, value)
        })
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        self.config.set_str(name, value).chain_err(|| {
            format!("error setting git config '{}': '{}'", name, value)
        })
    }
}
