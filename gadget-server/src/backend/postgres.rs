use std::convert::TryInto;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::{Pool, PooledConnection};

use super::models::*;
use super::schema::redirects::dsl::*;
use super::RowChange;
use diesel::prelude::*;

pub struct PostgresBackend {
    connection: PooledConnection<ConnectionManager<PgConnection>>,
}

impl Into<RowChange<usize>> for QueryResult<usize> {
    fn into(self) -> RowChange<usize> {
        match self {
            Ok(0) => RowChange::NotFound,
            Ok(i) => RowChange::Value(i),
            Err(e) => RowChange::Err(format!("{}", e)),
        }
    }
}

impl Into<RowChange<RedirectModel>> for QueryResult<RedirectModel> {
    fn into(self) -> RowChange<RedirectModel> {
        match self {
            Ok(i) => RowChange::Value(i),
            Err(diesel::result::Error::NotFound) => RowChange::NotFound,
            Err(e) => RowChange::Err(format!("{}", e)),
        }
    }
}

impl PostgresBackend {
    pub fn new<S: ToString>(connection: S) -> Self {
        info!("Connecting to PostgresDB");
        let manager = ConnectionManager::<PgConnection>::new(&connection.to_string());
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(true)
            .build(manager)
            .unwrap();
        let conn = pool.get().unwrap();
        PostgresBackend { connection: conn }
    }
}

impl super::Backend for PostgresBackend {
    fn get_redirect(&self, redirect_ref: &str) -> RowChange<RedirectModel> {
        redirects
            .filter(alias.eq(redirect_ref))
            .or_filter(public_ref.eq(redirect_ref))
            .get_result::<RedirectModel>(&self.connection)
            .into()
    }

    fn create_redirect(&self, new_alias: &str, new_destination: &str) -> RowChange<RedirectModel> {
        let new_redirect = RedirectInsert::new(new_alias, new_destination);

        match diesel::insert_into(redirects)
            .values(&new_redirect)
            .get_result::<RedirectModel>(&self.connection)
        {
            Ok(value) => RowChange::Value(value),
            Err(e) => RowChange::Err(format!("{:?}", e)),
        }
    }

    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize> {
        let filter = redirects
            .filter(alias.eq(redirect_ref))
            .or_filter(public_ref.eq(redirect_ref));

        diesel::delete(filter).execute(&self.connection).into()
    }

    fn update_redirect(&self, redirect_ref: &str, new_dest: &str) -> RowChange<usize> {
        let filter = redirects
            .filter(alias.eq(redirect_ref))
            .or_filter(public_ref.eq(redirect_ref));

        diesel::update(filter)
            .set(destination.eq(new_dest))
            .execute(&self.connection)
            .into()
    }

    fn get_all(&self, page: u64, limit: usize) -> Result<Vec<RedirectModel>, String> {
        let i_limit: i64 = match limit.try_into() {
            Ok(i) => i,
            Err(e) => {
                return Err(format!("{}", e));
            }
        };

        let i_page: i64 = match page.try_into() {
            Ok(i) => i,
            Err(e) => {
                return Err(format!("{}", e));
            }
        };

        redirects
            .limit(i_limit)
            .offset(i_page * i_limit)
            .get_results::<RedirectModel>(&self.connection)
            .map_err(|x| format!("{:?}", x))
    }
}

unsafe impl Sync for PostgresBackend {}
