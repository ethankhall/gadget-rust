use std::collections::BTreeMap;
use std::boxed::Box;

use super::{DataSource, DataSourceError};
use super::memory::InternalDataStore;

use bson::Bson;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::coll::Collection;


pub struct MongoDataSource {
    client: Box<Client>,
    current_results: InternalDataStore,
}

impl MongoDataSource {
    pub fn new(uri: &str) -> Result<MongoDataSource, DataSourceError> {
        let client = match ThreadedClient::with_uri(uri) {
            Ok(value) => value,
            Err(err) => return Err(DataSourceError::new(err))
        };

        let client = Box::new(client);

        let datasource = MongoDataSource { client, current_results: InternalDataStore::new() };
        return match datasource.reload() {
            Ok(_) => Ok(datasource),
            Err(message) => Err(message)
        };
    }
}

impl DataSource for MongoDataSource {
    fn retrieve_lookup(&self, name: String) -> Option<String> {
        return self.current_results.retrieve_lookup(name);
    }

    fn reload(&self) -> Result<(), DataSourceError> {
        let coll: Collection = self.client.db("gogo-gadget").collection("redirects");
        let results = match coll.find(None, None) {
            Err(err) => return Err(DataSourceError::new(err)),
            Ok(value) => value
        };

        let mut new_map: BTreeMap<String, String> = BTreeMap::new();

        for result in results {
            if let Ok(item) = result {
                if let Some(&Bson::String(ref alias)) = item.get("alias") {
                    if let Some(&Bson::String(ref redirect)) = item.get("redirect") {
                        new_map.insert(alias.clone(), redirect.clone());
                    }
                }
            }
        }

        self.current_results.update(new_map);

        return Ok(());
    }

    fn add_new_redirect(&self, alias: &str, redirect: &str) -> Result<(), DataSourceError> {
        let coll: Collection= self.client.db("gogo-gadget").collection("redirects");

        return match coll.insert_one(doc!{ "alias": alias, "redirect": redirect }, None) {
            Ok(_) => Ok(()),
            Err(err) => Err(DataSourceError::new(err))
        };
    }
}