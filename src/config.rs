use std::collections::HashMap;
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
