use std::collections::HashMap;
use errors::*;

pub trait Config {
    fn get(&self, name: &str) -> Result<String>;
    fn get_all(&self, glob: &str) -> Result<HashMap<String, String>>;
    fn add(&mut self, name: &str, value: &str) -> Result<()>;
    fn set(&mut self, name: &str, value: &str) -> Result<()>;
    fn clear(&mut self, name: &str) -> Result<()>;
}
