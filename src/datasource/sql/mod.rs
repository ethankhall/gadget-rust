use datasource::sql::models::{NewRedirects, Redirects};
use diesel::Connection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use self::schema::redirects::dsl::*;
use std::collections::BTreeMap;
use std::str::FromStr;
use super::{DataSource, DataSourceError};
use super::memory::InternalDataStore;

mod models;
mod schema;

#[derive(Debug)]
pub enum DbDriver {
    SQLite
}

impl DbDriver {
    pub fn variants<'b>() -> &'b [&'b str] {
        return &["sqlite"];
    }
}

impl FromStr for DbDriver {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        return match s.to_ascii_lowercase().as_str() {
            "sqlite" => Ok(DbDriver::SQLite),
            _ => Err(format!("Unable to parse {} into DbDriver", s))
        };
    }
}

enum SqlConnection {
    SQLite(SqliteConnection)
}

pub struct SqlDataSource {
    connection: SqlConnection,
    current_results: InternalDataStore,
}

impl SqlDataSource {
    pub fn new(path: &str, driver: DbDriver) -> Result<SqlDataSource, DataSourceError> {
        let connection = match driver {
            DbDriver::SQLite => {
                match SqliteConnection::establish(path) {
                    Ok(con) => SqlConnection::SQLite(con),
                    Err(msg) => {
                        return Err(DataSourceError::new(msg));
                    }
                }
            }
        };

        let datasource = SqlDataSource { connection, current_results: InternalDataStore::new() };
        return match datasource.reload() {
            Ok(_) => Ok(datasource),
            Err(message) => Err(message)
        };
    }
}

impl DataSource for SqlDataSource {
    fn retrieve_lookup(&self, name: String) -> Option<String> {
        return self.current_results.retrieve_lookup(name);
    }

    fn reload(&self) -> Result<(), DataSourceError> {
        let connection = match self.connection {
            SqlConnection::SQLite(ref con) => con
        };

        let requests = match redirects.load::<Redirects>(connection) {
            Ok(values) => values,
            Err(err) => {
                return Err(DataSourceError::new(err));
            }
        };

        let mut new_redirects: BTreeMap<String, String> = BTreeMap::new();

        for request in requests {
            new_redirects.insert(request.alias, request.destination);
        };

        self.current_results.update(new_redirects);
        return Ok(());
    }

    fn add_new_redirect(&self, alias_name: String, redirect: String) -> Result<(), DataSourceError> {
        use self::schema::redirects;

        let redirect = NewRedirects {
            alias: &alias_name,
            destination: &redirect,
            created_by: "Unknown",
        };

        let connection = match self.connection {
            SqlConnection::SQLite(ref con) => con
        };

        let insert = diesel::insert_into(redirects::table)
            .values(&redirect)
            .execute(connection);

        return match insert {
            Ok(_) => Ok(()),
            Err(err) => Err(DataSourceError::new(err))
        };
    }
}