use crate::models::ApiUser;
use std::convert::Infallible;
use thiserror::Error;
#[cfg(feature = "oauth2-proxy")]
use tracing::error;
use warp::Filter;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum UserError {
    #[cfg(any(feature = "oauth2-proxy"))]
    #[error("No 'x-forwarded-user' header present")]
    NoXForwardedUser,
    #[cfg(any(feature = "oauth2-proxy"))]
    #[error("No 'x-forwarded-preferred-username' header present")]
    NoXForwardedPreferredUsername,
    #[cfg(any(feature = "oauth2-proxy"))]
    #[error("Unable to fetch value from authorization header. {error}")]
    ToStringError {
        error: warp::http::header::ToStrError,
    },
}

#[cfg(not(any(feature = "oauth2-proxy", feature = "allow-no-auth")))]
compile_error!(
    "At least one auth config must be enabled. Avaliable options are 'oauth2-proxy', 'allow-no-auth' "
);

pub fn extract_user() -> impl Filter<Extract = (Option<ApiUser>,), Error = Infallible> + Clone {
    warp::filters::header::headers_cloned().map(|headers| {
        if cfg!(test) {
            return Some(ApiUser {
                id: format!("1234"),
                name: format!("test user"),
            });
        };

        let no_proxy_user = if cfg!(feature = "allow-no-auth") {
            return Some(ApiUser {
                id: format!("1234"),
                name: format!("Unknown"),
            });
        } else {
            None
        };

        let oauth2_user = if cfg!(feature = "oauth2-proxy") {
            match oauth2_proxy::extract_user(&headers) {
                Ok(user) => Some(user),
                Err(e) => {
                    error!("Unable to find user. {}", e.to_string());
                    None
                }
            }
        } else {
            None
        };

        oauth2_user.or(no_proxy_user)
    })
}

#[cfg(feature = "oauth2-proxy")]
mod oauth2_proxy {
    use super::{ApiUser, UserError};
    use tracing::trace;
    use warp::http::{HeaderMap, HeaderValue};

    fn extract_string(value: &HeaderValue) -> Result<String, UserError> {
        match value.to_str() {
            Ok(value) => Ok(value.to_owned()),
            Err(e) => Err(UserError::ToStringError { error: e }),
        }
    }

    pub fn extract_user(headers: &HeaderMap) -> Result<ApiUser, UserError> {
        trace!("Headers: {:?}", headers.keys());

        let x_forwarded_user = headers
            .get("x-forwarded-user")
            .ok_or(UserError::NoXForwardedUser)
            .and_then(extract_string);
        let x_forwarded_preferrred_username = headers
            .get("x-forwarded-preferred-username")
            .ok_or(UserError::NoXForwardedPreferredUsername)
            .and_then(extract_string);

        match (x_forwarded_user, x_forwarded_preferrred_username) {
            (Ok(id), Ok(name)) => Ok(ApiUser { id, name }),
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    }
}
