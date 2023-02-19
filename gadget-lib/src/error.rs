use thiserror::Error;

#[derive(Error, Debug)]
pub enum GadgetLibError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Unable to fetch internal state `{0}`")]
    PoisonError(String),
    #[error("Redirect {0} already exists")]
    RedirectExists(String),
    #[error("Redirect {0} does not exists")]
    RedirectDoesNotExists(String),
    #[error("Unknown backend for {0}")]
    UnknownBackend(String),
}

impl<T> From<std::sync::PoisonError<T>> for GadgetLibError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        GadgetLibError::PoisonError(e.to_string())
    }
}
