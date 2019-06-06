use std::path::PathBuf;
use std::error::Error;

pub mod compile;
use compile::*;

pub fn read_config(path: PathBuf) -> Result<ConfigRoot, String> {
    match std::fs::read_to_string(path) {
        Ok(body) => {
            match serde_yaml::from_str(&body) {
                Ok(root) => Ok(root),
                Err(err) => Err(s!(err.description()))
            }
        },
        Err(err) => Err(s!(err.description()))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigRoot {
    pub missing_redirect_destination: String,
    pub redirects: Vec<RedirectDefinition>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedirectDefinition {
    pub alias: String,
    pub destination: String
}

impl ConfigRoot {
    pub fn compile(self) -> CompiledConfigs {
        CompiledConfigs::new(self.missing_redirect_destination, self.redirects)
    }
}