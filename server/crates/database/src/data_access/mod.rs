use async_trait::async_trait;
use thiserror::Error;
use tracing::{error, info};
use tracing_attributes::instrument;

use crate::entity::{external_user, global_redirects, prelude::*};
use sea_orm::{
    entity::*, prelude::*, query::*, Condition, ConnectOptions, Database, DatabaseConnection,
    DatabaseTransaction,
};
use std::time::Duration;

pub mod model;
use self::model::*;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error(transparent)]
    SeaOrmError {
        #[from]
        source: sea_orm::DbErr,
    },
    #[error("Redirect not found matching `{search}`")]
    RedirectNotFound { search: String },
    #[error("User with id {id} not found")]
    UserNotFound { id: i32 },
    #[error("Redirect already exists for `{alias}`")]
    RedirectAlreadyExists { alias: String },
    #[error(transparent)]
    SqlxError {
        #[from]
        source: sqlx::Error,
    },
    #[error(transparent)]
    UnitConversionError {
        #[from]
        source: std::num::TryFromIntError,
    },
}

pub(crate) type DbResult<T> = Result<T, DatabaseError>;

pub struct PostgresDb {
    db: DatabaseConnection,
}

impl PostgresDb {
    pub async fn new(url: &str) -> DbResult<Self> {
        info!("Creating new database connection to {}", url);
        let mut opt = ConnectOptions::new(url.to_owned());
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true);

        let db: DatabaseConnection = Database::connect(opt).await?;

        Ok(PostgresDb { db })
    }

    async fn upsert_user(tx: &DatabaseTransaction, user: &DbUser) -> DbResult<i32> {
        let found_user = ExternalUser::find()
            .column(external_user::Column::UserId)
            .filter(external_user::Column::ExternalUserId.eq(user.external_id.clone()))
            .one(tx)
            .await?;

        let user_model = match found_user {
            Some(found_user) => {
                let mut found_user = found_user.into_active_model();
                found_user.prefered_name = Set(user.name.clone());
                found_user
            }
            None => external_user::ActiveModel {
                prefered_name: Set(user.name.clone()),
                external_user_id: Set(user.external_id.clone()),
                ..Default::default()
            },
        };

        let mut user = user_model.save(tx).await?;
        Ok(user.user_id.take().expect("User to have valid user_id"))
    }

    async fn get_user(tx: &DatabaseTransaction, user_id: i32) -> DbResult<DbUser> {
        let user = ExternalUser::find_by_id(user_id).one(tx).await?;
        match user {
            None => Err(DatabaseError::UserNotFound { id: user_id }),
            Some(value) => Ok((&value).into()),
        }
    }

    async fn find_redirect(
        tx: &DatabaseTransaction,
        redirect_ref: &str,
    ) -> DbResult<global_redirects::Model> {
        let results = GlobalRedirects::find()
            .filter(
                Condition::any()
                    .add(global_redirects::Column::PublicRef.eq(redirect_ref))
                    .add(global_redirects::Column::Alias.eq(redirect_ref)),
            )
            .one(tx)
            .await?;

        let redirect: global_redirects::Model = match results {
            Some(value) => value,
            None => {
                return Err(DatabaseError::RedirectNotFound {
                    search: redirect_ref.to_owned(),
                })
            }
        };

        Ok(redirect)
    }
}

#[async_trait]
pub trait DatabaseEngine {
    async fn get_redirect_by_id_or_alias(&self, redirect_ref: &str)
        -> DbResult<DbRedirectUserPair>;

    async fn create_redirect(
        &self,
        redirect: &DbNewRedirect,
        user: &DbUser,
    ) -> DbResult<DbRedirect>;

    async fn update_redirect(
        &self,
        redirect: &DbUpdateRedirect,
        user: &DbUser,
    ) -> DbResult<DbRedirect>;

    async fn delete_redirect(&self, redirect_ref: &str) -> DbResult<DbRedirectUserPair>;

    async fn get_all(&self, pagination: &PaginationOptions) -> DbResult<Vec<DbRedirectUserPair>>;

    async fn count(&self) -> DbResult<u32>;
}

#[async_trait]
impl DatabaseEngine for PostgresDb {
    #[instrument(skip(self))]
    async fn get_redirect_by_id_or_alias(
        &self,
        redirect_ref: &str,
    ) -> DbResult<DbRedirectUserPair> {
        let tx = self.db.begin().await?;
        let redirect = PostgresDb::find_redirect(&tx, redirect_ref).await?;
        let user = PostgresDb::get_user(&tx, redirect.created_by_user_id).await?;

        tx.commit().await?;
        Ok(DbRedirectUserPair::new((&redirect).into(), user))
    }

    #[instrument(skip(self))]
    async fn delete_redirect(&self, redirect_ref: &str) -> DbResult<DbRedirectUserPair> {
        let tx = self.db.begin().await?;
        let results = PostgresDb::find_redirect(&tx, redirect_ref).await?;
        let user_id = results.created_by_user_id;
        let deleted_results_model = (&results).into();
        results.delete(&tx).await?;

        let user = PostgresDb::get_user(&tx, user_id).await?;

        tx.commit().await?;

        Ok(DbRedirectUserPair::new(deleted_results_model, user))
    }

    #[instrument(skip(self))]
    async fn create_redirect(
        &self,
        redirect: &DbNewRedirect,
        user: &DbUser,
    ) -> DbResult<DbRedirect> {
        let tx = self.db.begin().await?;
        let count_of_matching_aliases = GlobalRedirects::find()
            .filter(global_redirects::Column::Alias.eq(redirect.alias.to_owned()))
            .count(&tx)
            .await?;

        if count_of_matching_aliases != 0 {
            return Err(DatabaseError::RedirectAlreadyExists {
                alias: redirect.alias.to_owned(),
            });
        }

        let user_id = PostgresDb::upsert_user(&tx, user).await?;

        let redirect_model = global_redirects::ActiveModel {
            public_ref: Set(redirect.public_ref.to_owned()),
            alias: Set(redirect.alias.to_owned()),
            destination: Set(redirect.destination.to_owned()),
            created_on: Set(redirect.created_on.naive_utc()),
            created_by_user_id: Set(user_id),
            ..Default::default()
        };

        let result = redirect_model.insert(&tx).await?;
        tx.commit().await?;

        Ok((&result).into())
    }

    #[instrument(skip(self))]
    async fn update_redirect(
        &self,
        redirect: &DbUpdateRedirect,
        user: &DbUser,
    ) -> DbResult<DbRedirect> {
        let tx = self.db.begin().await?;
        let user_id = PostgresDb::upsert_user(&tx, user).await?;

        let results = GlobalRedirects::find()
            .filter(global_redirects::Column::PublicRef.eq(redirect.public_ref.to_owned()))
            .one(&tx)
            .await?;

        let found_redirect = match results {
            Some(value) => value,
            None => {
                return Err(DatabaseError::RedirectNotFound {
                    search: redirect.public_ref.to_owned(),
                })
            }
        };

        let redirect_id = found_redirect.global_redirects_id;

        let mut active_model = found_redirect.into_active_model();

        active_model.destination = Set(redirect.destination.to_owned());
        active_model.created_on = Set(redirect.created_on.naive_utc());
        active_model.created_by_user_id = Set(user_id);

        active_model.save(&tx).await?;

        let results = GlobalRedirects::find_by_id(redirect_id).one(&tx).await?;

        let found_redirect = match results {
            Some(value) => value,
            None => {
                return Err(DatabaseError::RedirectNotFound {
                    search: redirect.public_ref.to_owned(),
                })
            }
        };

        tx.commit().await?;
        Ok((&found_redirect).into())
    }

    #[instrument(skip(self))]
    async fn get_all(&self, pagination: &PaginationOptions) -> DbResult<Vec<DbRedirectUserPair>> {
        let results = GlobalRedirects::find()
            .find_also_related(ExternalUser)
            .order_by_asc(global_redirects::Column::PublicRef)
            .paginate(&self.db, pagination.page_size.try_into()?)
            .fetch_page(pagination.page_number.try_into()?)
            .await?;

        let found_results = results
            .iter()
            .map(|(redirect, user)| {
                DbRedirectUserPair::new(
                    redirect.into(),
                    user.as_ref().expect("User should already exist").into(),
                )
            })
            .collect();

        Ok(found_results)
    }

    #[instrument(skip(self))]
    async fn count(&self) -> DbResult<u32> {
        let count = GlobalRedirects::find().count(&self.db).await?;

        Ok(u32::try_from(count)?)
    }
}

#[cfg(test)]
mod integ_test {
    use crate::prelude::*;
    use gadget_test_utils::db_utils::*;
    use serial_test::serial;

    async fn make_db() -> PostgresDb {
        setup_schema().await;
        PostgresDb::new(&get_db_url_with_test_db()).await.unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[serial]
    async fn crud_for_global_redirect() {
        let dao = make_db().await;

        let result = dao.get_redirect_by_id_or_alias("google").await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!("Redirect not found matching `google`", e.to_string());
        }

        let result = dao
            .create_redirect(
                &DbNewRedirect::new("google", "https://google.com"),
                &DbUser::new("11", "test"),
            )
            .await
            .unwrap();
        assert_eq!(result.alias, "google");
        assert_eq!(result.destination, "https://google.com");

        let result = dao
            .create_redirect(
                &DbNewRedirect::new("google", "https://duckduckgo.com"),
                &DbUser::new("11", "test"),
            )
            .await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!("Redirect already exists for `google`", e.to_string());
        }

        let pair = dao.get_redirect_by_id_or_alias("google").await.unwrap();
        assert_eq!(pair.redirect.alias, "google");
        assert_eq!(pair.redirect.destination, "https://google.com");
        assert_eq!(pair.user.external_id, "11");
        assert_eq!(pair.user.name, "test");

        let pair = dao.delete_redirect("google").await.unwrap();
        assert_eq!(pair.redirect.alias, "google");
        assert_eq!(pair.redirect.destination, "https://google.com");
        assert_eq!(pair.user.external_id, "11");
        assert_eq!(pair.user.name, "test");

        let result = dao.get_redirect_by_id_or_alias("google").await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!("Redirect not found matching `google`", e.to_string());
        }

        let result = dao
            .create_redirect(
                &DbNewRedirect::new("google", "https://google.com"),
                &DbUser::new("11", "test"),
            )
            .await
            .unwrap();
        assert_eq!(result.alias, "google");
        assert_eq!(result.destination, "https://google.com");

        let pair = dao
            .get_redirect_by_id_or_alias(&result.public_ref)
            .await
            .unwrap();
        assert_eq!(pair.redirect.alias, "google");
        assert_eq!(pair.redirect.destination, "https://google.com");
        assert_eq!(pair.user.external_id, "11");
        assert_eq!(pair.user.name, "test");

        let pair = dao.delete_redirect(&result.public_ref).await.unwrap();
        assert_eq!(pair.redirect.alias, "google");
        assert_eq!(pair.redirect.destination, "https://google.com");
        assert_eq!(pair.user.external_id, "11");
        assert_eq!(pair.user.name, "test");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[serial]
    async fn pagination_tests() {
        use std::collections::BTreeSet;

        // let _logger = logging_setup();
        let dao = make_db().await;

        let user = DbUser::new("11", "test");
        let mut inserted_elements: BTreeSet<String> = Default::default();

        for i in 0..100 {
            let result = dao
                .create_redirect(
                    &DbNewRedirect::new(&format!("request-{}", i), &format!("{}", i)),
                    &user,
                )
                .await
                .unwrap();
            inserted_elements.insert(result.public_ref);
        }

        assert_eq!(dao.count().await.unwrap(), 100);

        for page_number in 0..10 {
            let page = dao
                .get_all(&PaginationOptions::new(page_number, 10))
                .await
                .unwrap();
            assert_eq!(page.len(), 10);
            for redirect in page {
                assert!(inserted_elements.remove(&redirect.redirect.public_ref));
            }
            assert_eq!(inserted_elements.len() as u32, 100 - (page_number + 1) * 10);
        }
        assert_eq!(inserted_elements.len(), 0);

        let page = dao.get_all(&PaginationOptions::new(11, 10)).await.unwrap();
        assert_eq!(page.len(), 0);
    }
}
