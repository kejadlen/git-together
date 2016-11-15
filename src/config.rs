use std::collections::HashMap;
use git2;
use errors::*;

pub trait Config {
  fn get(&self, name: &str) -> Result<String>;
  fn get_all(&self, glob: &str) -> Result<HashMap<String, String>>;
  fn add(&mut self, name: &str, value: &str) -> Result<()>;
  fn set(&mut self, name: &str, value: &str) -> Result<()>;
}

pub struct NamespacedConfig<C> {
  namespace: String,
  config: C,
}

impl<C> NamespacedConfig<C> {
  pub fn new(namespace: &str, config: C) -> Self {
    NamespacedConfig {
      namespace: namespace.into(),
      config: config,
    }
  }

  fn namespaced(&self, name: &str) -> String {
    format!("{}.{}", self.namespace, name)
  }
}

impl<C: Config> Config for NamespacedConfig<C> {
  fn get(&self, name: &str) -> Result<String> {
    self.config.get(&self.namespaced(name))
  }

  fn get_all(&self, glob: &str) -> Result<HashMap<String, String>> {
    self.config.get_all(&self.namespaced(glob))
  }

  fn add(&mut self, name: &str, value: &str) -> Result<()> {
    let name = self.namespaced(name);
    self.config.add(&name, value)
  }

  fn set(&mut self, name: &str, value: &str) -> Result<()> {
    let name = self.namespaced(name);
    self.config.set(&name, value)
  }
}

pub struct GitConfig {
  pub config: git2::Config,
}

impl Config for GitConfig {
  fn get(&self, name: &str) -> Result<String> {
    self.config
      .get_string(name)
      .chain_err(|| format!("error getting git config for '{}'", name))
  }

  fn get_all(&self, glob: &str) -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();
    let entries = self.config
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
}
