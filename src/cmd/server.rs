use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use anyhow::Result;
use clap::ArgMatches;

async fn hello(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!")))
}

pub async fn execute(args: &ArgMatches) -> Result<()> {
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });
    let addr = ([127, 0, 0, 1], args.value_of_t("port")?).into();

    let server = Server::bind(&addr).serve(make_svc);

    tracing::info!("docse {}", addr);

    server.await.map_err(Into::into)
}
