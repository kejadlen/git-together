use std::collections::HashMap;
use std::env;

use git2;

use config;
use errors::*;
use ConfigScope;

pub struct Repo {
    repo: git2::Repository,
}

impl Repo {
    pub fn new() -> Result<Self> {
        let repo = env::current_dir()
            .chain_err(|| "")
            .and_then(|current_dir| git2::Repository::discover(current_dir).chain_err(|| ""))?;
        Ok(Repo { repo })
    }

    pub fn config(&self) -> Result<Config> {
        self.repo
            .config()
            .map(|config| Config { config })
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
            .chain_err(|| "")?
            .into_iter()
            .map(|entry| {
                entry
                    .chain_err(|| "")
                    .and_then(|entry| entry.value().map(String::from).ok_or_else(|| "".into()))
            })
            .collect::<Result<_>>()?;
        Ok(include_paths)
    }

    fn local_config(&self) -> Result<git2::Config> {
        let config = self.repo.config().chain_err(|| "")?;
        config.open_level(git2::ConfigLevel::Local).chain_err(|| "")
    }
}

pub struct Config {
    config: git2::Config,
}

impl Config {
    pub fn new(scope: ConfigScope) -> Result<Self> {
        let config = match scope {
            ConfigScope::Local => git2::Config::open_default(),
            ConfigScope::Global => git2::Config::open_default().and_then(|mut r| r.open_global()),
        };

        config.map(|config| Config { config }).chain_err(|| "")
    }
}

impl config::Config for Config {
    fn get(&self, name: &str) -> Result<String> {
        self.config
            .get_string(name)
            .chain_err(|| format!("error getting git config for '{}'", name))
    }

    fn get_all(&self, glob: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        let entries = self
            .config
            .entries(Some(glob))
            .chain_err(|| "error getting git config entries")?;
        for entry in &entries {
            let entry = entry.chain_err(|| "error getting git config entry")?;
            if let (Some(name), Some(value)) = (entry.name(), entry.value()) {
                result.insert(name.into(), value.into());
            }
        }
        Ok(result)
    }

    fn add(&mut self, name: &str, value: &str) -> Result<()> {
        self.config
            .set_multivar(name, "^$", value)
            .chain_err(|| format!("error adding git config '{}': '{}'", name, value))
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        self.config
            .set_str(name, value)
            .chain_err(|| format!("error setting git config '{}': '{}'", name, value))
    }

    fn clear(&mut self, name: &str) -> Result<()> {
        self.config
            .remove(name)
            .chain_err(|| format!("error removing git config '{}'", name))
    }
}
