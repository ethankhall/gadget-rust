use serde_yaml;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use super::{DataSource, DataSourceError};
use super::memory::InternalDataStore;


pub struct YamlDataSource {
    path: PathBuf,
    current_results: InternalDataStore,
}

impl YamlDataSource {
    pub fn new(path: PathBuf) -> Result<YamlDataSource, DataSourceError> {
        let datasource = YamlDataSource { path, current_results: InternalDataStore::new() };
        return match datasource.reload() {
            Ok(_) => Ok(datasource),
            Err(message) => Err(message)
        };
    }

    fn reload_from_string(&self, contents: String) -> Result<(), DataSourceError> {
        let document: YamlDatasourceDocument = match serde_yaml::from_str(&contents) {
            Ok(val) => val,
            Err(err) => { return Err(DataSourceError::new(err)); }
        };

        self.current_results.update(document.definitions);

        return Ok(());
    }
}

fn read_file_to_string(path: &Path) -> Result<String, DataSourceError> {
    return match File::open(path) {
        Ok(mut file) => {
            let mut contents = String::new();
            return match file.read_to_string(&mut contents) {
                Ok(_) => Ok(contents),
                Err(value) => Err(DataSourceError::new(value.to_string()))
            };
        }
        Err(value) => Err(DataSourceError::new(value))
    };
}

impl DataSource for YamlDataSource {
    fn retrieve_lookup(&self, name: String) -> Option<String> {
        return self.current_results.retrieve_lookup(name);
    }

    fn reload(&self) -> Result<(), DataSourceError> {
        let contents = match read_file_to_string(self.path.as_path()) {
            Ok(val) => val,
            Err(err) => { return Err(err); }
        };

        return self.reload_from_string(contents);
    }

    fn add_new_redirect(&self, _alias: String, _redirect: String) -> Result<(), DataSourceError> {
        return Err(DataSourceError::new("YAML backend doesn't support writing."));
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct YamlDatasourceDocument {
    definitions: BTreeMap<String, String>
}

#[test]
fn will_parse_sample_yaml() {
    let sample_yaml = "---\n
definitions:\n
    lookup: foo\n
    bank: www.google.com";

    let datasource = YamlDataSource { path: PathBuf::from("/tmp/yaml"), current_results: InternalDataStore::new() };
    assert!(datasource.reload_from_string(String::from(sample_yaml)).is_ok());
    assert_eq!(Some(String::from("foo")), datasource.retrieve_lookup(String::from("lookup")));
    assert_eq!(Some(String::from("www.google.com")), datasource.retrieve_lookup(String::from("bank")));
}