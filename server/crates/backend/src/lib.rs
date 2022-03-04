use thiserror::Error;
use tracing::error;
use tracing_attributes::instrument;

use gadget_database::prelude::*;
pub mod models;

use models::{BackendPaginationOptions, BackendRedirect, BackendRedirectList, BackendRedirectUser};

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Redirect not found matching `{search}`")]
    RedirectNotFound { search: String },
    #[error("User with id {id} not found")]
    UserNotFound { id: i32 },
    #[error("Redirect already exists for `{alias}`")]
    RedirectAlreadyExists { alias: String },
    #[error("Unable to convert number: {message}")]
    InvalidNumber { message: String },
    #[error("Internal Server Error")]
    SqlError { error: anyhow::Error },
}

impl From<DatabaseError> for BackendError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::SeaOrmError { source } => BackendError::SqlError {
                error: source.into(),
            },
            DatabaseError::RedirectNotFound { search } => BackendError::RedirectNotFound { search },
            DatabaseError::UserNotFound { id } => BackendError::UserNotFound { id },
            DatabaseError::RedirectAlreadyExists { alias } => {
                BackendError::RedirectAlreadyExists { alias }
            }
            DatabaseError::SqlxError { source } => BackendError::SqlError {
                error: source.into(),
            },
            DatabaseError::UnitConversionError { source } => BackendError::InvalidNumber {
                message: source.to_string(),
            },
        }
    }
}

pub struct DefaultBackend {
    database: PostgresDb,
}

impl DefaultBackend {
    pub async fn new(url: &str) -> Result<Self, BackendError> {
        Ok(Self {
            database: gadget_database::prelude::PostgresDb::new(url).await?,
        })
    }

    #[instrument(skip(self))]
    pub async fn get_redirect(&self, redirect_ref: &str) -> Result<BackendRedirect, BackendError> {
        let pair = self
            .database
            .get_redirect_by_id_or_alias(redirect_ref)
            .await?;
        Ok((&pair).into())
    }

    #[instrument(skip(self))]
    pub async fn create_redirect(
        &self,
        alias: &str,
        destination: &str,
        user: &BackendRedirectUser,
    ) -> Result<BackendRedirect, BackendError> {
        let db_user = &user.into();
        let redirect = self
            .database
            .create_redirect(&DbNewRedirect::new(alias, destination), db_user)
            .await?;

        Ok(BackendRedirect::from_db_models(&redirect, db_user))
    }

    #[instrument(skip(self))]
    pub async fn update_redirect(
        &self,
        redirect_ref: &str,
        destination: &str,
        user: &BackendRedirectUser,
    ) -> Result<BackendRedirect, BackendError> {
        let db_user = &user.into();
        let redirect = self
            .database
            .update_redirect(&DbUpdateRedirect::new(redirect_ref, destination), db_user)
            .await?;

        Ok(BackendRedirect::from_db_models(&redirect, db_user))
    }

    #[instrument(skip(self))]
    pub async fn delete_redirect(
        &self,
        redirect_ref: &str,
    ) -> Result<BackendRedirect, BackendError> {
        let pair = self.database.delete_redirect(redirect_ref).await?;

        Ok((&pair).into())
    }

    #[instrument(skip(self))]
    pub async fn get_all(
        &self,
        pagination: BackendPaginationOptions,
    ) -> Result<BackendRedirectList, BackendError> {
        let db_pagination = (&pagination).into();
        let page = self.database.get_all(&db_pagination).await?;
        let count = self.database.count().await?;

        let redirects: Vec<BackendRedirect> = page.iter().map(|pair| pair.into()).collect();
        Ok(BackendRedirectList::from(
            redirects,
            count,
            pagination.has_more(count),
        ))
    }
}

pub mod prelude {
    pub use crate::models::*;
    pub use crate::{BackendError, DefaultBackend};
}
