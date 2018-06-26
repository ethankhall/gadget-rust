use std::path::{PathBuf, Path};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::cell::RefCell;

use serde_yaml;

use super::{DataSource, DataSourceError};


pub struct YamlDataSource {
    path: PathBuf,
    current_results: Arc<RefCell<BTreeMap<String, String>>>,
}

impl YamlDataSource {
    pub fn new(path: PathBuf) -> Result<YamlDataSource, DataSourceError> {
        let datasource = YamlDataSource { path, current_results: Arc::new(RefCell::new(BTreeMap::new())) };
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

        self.current_results.replace(document.definitions);

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
        return match self.current_results.try_borrow() {
            Ok(val) => {
                return match val.get(name.as_str()) {
                    Some(result) => Some(format!("{}", result)),
                    None => None
                };
            }
            Err(_) => None
        };
    }

    fn reload(&self) -> Result<(), DataSourceError> {
        let contents = match read_file_to_string(self.path.as_path()) {
            Ok(val) => val,
            Err(err) => { return Err(err); }
        };

        return self.reload_from_string(contents);
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

    let datasource = YamlDataSource { path: PathBuf::from("/tmp/yaml"), current_results: Arc::new(RefCell::new(BTreeMap::new())) };
    assert!(datasource.reload_from_string(String::from(sample_yaml)).is_ok());
    assert_eq!(Some(String::from("foo")), datasource.retrieve_lookup(String::from("lookup")));
    assert_eq!(Some(String::from("www.google.com")), datasource.retrieve_lookup(String::from("bank")));
}

unsafe impl Sync for YamlDataSource {

}

unsafe impl Send for YamlDataSource {

}