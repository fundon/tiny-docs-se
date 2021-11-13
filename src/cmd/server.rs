//! ```
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控"}'
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "kind": "h1" }'
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "kind": "t" }'
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "kind": "s" }'
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "kind": "p" }'
//! ```

use std::convert::Infallible;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{body::to_bytes, Body, Request, Response, Server, StatusCode};
use serde::Deserialize;

use anyhow::Result;
use clap::ArgMatches;

use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

#[derive(Deserialize)]
struct Data {
    search: Option<String>,
    kind: Option<String>,
}

async fn hello(
    req: Request<Body>,
    pool: r2d2::Pool<SqliteConnectionManager>,
) -> Result<Response<Body>> {
    let data: Data = serde_json::from_slice(&to_bytes(req.into_body()).await?)?;
    let search = data.search.filter(|v| !v.is_empty());
    let kind = data.kind.unwrap_or_else(|| "t".to_string());

    let builder = Response::builder().header("Content-Type", "application/json");

    if search.is_none() {
        return builder
            .body(Body::from(serde_json::to_vec(&serde_json::json!([]))?))
            .map_err(Into::into);
    }

    let conn = pool.get()?;
    let mut stmt = conn.prepare(r#"SELECT kind, simple_snippet(d, 2, '[', ']', '...', 10) from d where content match simple_query(?1) AND kind = ?2"#)?;
    let rows: Vec<(String, String)> = stmt
        .query_map([search.unwrap(), kind], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map(|rows| rows.filter_map(Result::ok).collect::<Vec<_>>())?;

    builder
        .body(Body::from(serde_json::to_vec(&serde_json::json!(rows))?))
        .map_err(Into::into)
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
