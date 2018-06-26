use std::marker::{Sync, Send};

pub mod yaml;

pub trait DataSource {
    fn retrieve_lookup(&self, name: String) -> Option<String>;
    fn reload(&self) -> Result<(), DataSourceError>;
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
}

unsafe impl Sync for DataSourceContainer {

}

unsafe impl Send for DataSourceContainer {

}



