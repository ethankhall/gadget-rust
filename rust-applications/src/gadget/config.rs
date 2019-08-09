use std::error::Error;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

pub fn read_config<T>(path: PathBuf) -> Result<T, String>
where
    T: DeserializeOwned,
{
    match std::fs::read_to_string(path) {
        Ok(body) => match serde_yaml::from_str(&body) {
            Ok(root) => Ok(root),
            Err(err) => Err(s!(err.description())),
        },
        Err(err) => Err(s!(err.description())),
    }
}
