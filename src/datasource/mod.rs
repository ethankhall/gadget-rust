use std::marker::{Send, Sync};

pub mod yaml;
pub mod sql;
mod memory;

pub trait DataSource {
    fn retrieve_lookup(&self, name: String) -> Option<String>;
    fn reload(&self) -> Result<(), DataSourceError>;
    fn add_new_redirect(&self, alias: String, redirect: String) -> Result<(), DataSourceError>;
}

#[derive(Debug)]
pub struct DataSourceError {
    message: String
}

impl DataSourceError {
    pub fn new<S: ToString>(message_body: S) -> Self {
        return DataSourceError { message: message_body.to_string() };
    }
}

pub struct DataSourceContainer {
    pub data_source: Box<DataSource>
}

impl DataSource for DataSourceContainer {
    fn retrieve_lookup(&self, name: String) -> Option<String> {
        return self.data_source.retrieve_lookup(name);
    }

    fn reload(&self) -> Result<(), DataSourceError> {
        return self.data_source.reload();
    }

    fn add_new_redirect(&self, alias: String, redirect: String) -> Result<(), DataSourceError> {
        return self.data_source.add_new_redirect(alias, redirect);
    }
}

unsafe impl Sync for DataSourceContainer {

}

unsafe impl Send for DataSourceContainer {

}



