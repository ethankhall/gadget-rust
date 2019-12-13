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
            Err(e) => RowChange::Err(format!("{}", e)),
        }
    }
}

impl PostgresBackend {
    pub fn new<S: ToString>(connection: S) -> Self {
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

    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
    ) -> RowChange<RedirectModel> {
        let new_redirect = RedirectInsert::new(new_alias, new_destination);

        match diesel::insert_into(redirects)
            .values(&new_redirect)
            .get_result::<RedirectModel>(&self.connection) {
                Ok(value) => RowChange::Value(value),
                Err(e) => RowChange::Err(format!("{:?}", e))
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

    fn get_all(&self, page: i64, limit: i64) -> Result<Vec<RedirectModel>, String> {
        redirects
            .limit(limit)
            .offset(page * limit)
            .get_results::<RedirectModel>(&self.connection)
            .map_err(|x| format!("{:?}", x))
    }
}

unsafe impl Sync for PostgresBackend {}
