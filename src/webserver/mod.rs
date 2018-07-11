use iron::prelude::*;
use router::Router;
use std::sync::Arc;
use super::datasource::DataSourceContainer;

mod redirect;
mod gadget;
mod util;

#[derive(RustEmbed)]
#[folder = "resources/public/"]
struct Asset;

pub fn exec_webserver(listen: &str, port: u32, datasource: Arc<DataSourceContainer>) {
    let redirect_handler = redirect::RedirectRequestHandler::new(datasource.clone());
    let gadget_get_handler = gadget::GadgetGetRequestHandler::new();
    let gadget_post_handler = gadget::GadgetPostRequestHandler::new(datasource);

    let mut router = Router::new();
    router.get("/gadget", gadget_get_handler, "gadget_base");
    router.post("/gadget/route", gadget_post_handler, "gadget_post");
    router.get("/:redirect", redirect_handler, "redirect");

    let mut chain = Chain::new(router);
    chain.around(util::Custom404);

    let listen_addr = format!("{listen}:{port}", port = port, listen = listen);
    info!("Listening on address {}", listen_addr);
    Iron::new(chain).http(listen_addr).unwrap();
}
