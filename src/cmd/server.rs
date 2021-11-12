use std::convert::Infallible;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use anyhow::Result;
use clap::ArgMatches;

use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

async fn hello(
    _req: Request<Body>,
    pool: r2d2::Pool<SqliteConnectionManager>,
) -> Result<Response<Body>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(r#"SELECT kind, simple_snippet(d, 2, '[', ']', '...', 10) from d where content match simple_query('数据') AND kind != 'p'"#)?;
    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map(|rows| rows.filter_map(Result::ok).collect::<Vec<_>>())?;

    Ok(Response::new(Body::from(serde_json::to_vec(
        &serde_json::json!(rows),
    )?)))
}

pub async fn execute(args: &ArgMatches) -> Result<()> {
    let root = std::env::current_dir()?;
    let manager = SqliteConnectionManager::file("docs.db").with_init(move |conn| {
        // let manager = SqliteConnectionManager::memory().with_init(move |conn| {
        unsafe {
            let path = root.join("libsimple.dylib");
            tracing::trace!("{:?}", path);
            conn.load_extension(path, None)?;
        }
        /*
        conn.execute_batch(
        r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS d USING fts5(id, parent, content, kind, uuid, version, locale, tokenize = 'simple');
            INSERT INTO d SELECT * FROM docs;
        "#)?;
        */
        Ok(())
    });
    let pool = r2d2::Pool::new(manager)?;

    let make_svc = make_service_fn(move |_conn| {
        let pool = pool.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let pool = pool.clone();
                async move { hello(req, pool).await }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], args.value_of_t("port")?).into();

    let server = Server::bind(&addr).serve(make_svc);

    tracing::info!("docse {}", addr);

    server.await.map_err(Into::into)
}
