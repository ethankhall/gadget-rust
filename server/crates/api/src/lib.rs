mod endpoints;
mod extractors;
mod models;
mod redirect;
mod static_response;
#[cfg(test)]
mod test;

use gadget_backend::prelude::*;
use std::sync::Arc;
use warp::{reply::Reply, Filter};

pub type SharedContext = Arc<SharedData>;

pub struct SharedData {
    pub backend: DefaultBackend,
    pub ui_location: url::Url,
}

pub async fn make_api_filters(
    backend: SharedContext,
) -> impl Filter<Extract = impl Reply> + Clone + Send + Sync + 'static {
    endpoints::api_endpoint(backend)
        .recover(static_response::handle_rejection)
        .with(warp::trace::request())
}

pub mod prelude {
    pub use crate::{endpoints::metrics_endpoint, make_api_filters, SharedContext, SharedData};
}
