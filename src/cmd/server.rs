//! ```
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "locale": "zh", "version": "v2.0"}'
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "locale": "zh", "version": "v2.0"}'
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "locale": "zh", "version": "v2.0"}'
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "locale": "zh", "version": "v2.0"}'
//! curl -X POST 127.0.0.1:3000 -H "Content-Type: application/json" -d '{"search": "监控", "locale": "zh", "version": "v2.0"}'
//! ```

use std::convert::Infallible;

use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::to_bytes, Body, Request, Response, Server};
use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    search: Option<String>,
    locale: Option<String>,
    version: Option<String>,
    p: Option<u64>,
    l: Option<u64>,
}

async fn hello(
    req: Request<Body>,
    pool: r2d2::Pool<SqliteConnectionManager>,
) -> Result<Response<Body>> {
    let data: Data = serde_json::from_slice(&to_bytes(req.into_body()).await?)?;
    let search = data.search.filter(|v| !v.is_empty());
    let locale = data.locale.unwrap_or_else(|| "cn".to_string());
    let version = data.version.unwrap_or_else(|| "v1.0".to_string());
    let p = data.p.unwrap_or_default().max(1);
    let l = data.l.unwrap_or_default();

    tracing::info!(
        "search = {:?}, locale = {}, version = {}, page = {}, page_size = {}",
        search,
        locale,
        version,
        p,
        l
    );

    let builder = Response::builder().header("Content-Type", "application/json");

    if search.is_none() {
        return builder
            .body(Body::from(serde_json::to_vec(&serde_json::json!([]))?))
            .map_err(Into::into);
    }

    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        r#"
    SELECT DISTINCT d.gid,
        d.tag,
        simple_snippet(d, 6, '[', ']', '...', 10) as content,
        ifnull(a.tag, 0) ptag,
        ifnull(a.content, '') pcontent,
        b.content title
    FROM d
    LEFT JOIN (
        SELECT id, tag, content FROM d
    ) AS a ON (d.tag = 7 AND d.pid = a.id)
    LEFT JOIN (
        SELECT id, gid, tag, content FROM d
    ) AS b on (b.tag = 1 and d.gid = b.gid)
    WHERE
        d.locale = ?1
    AND
        d.version = ?2
    AND
        d.content match simple_query(?3)
    ORDER BY
        d.gid,
        d.tag,
        d.rank
    LIMIT ?4
    OFFSET ?5
    "#,
    )?;
    let rows: Vec<(String, u8, String, u8, String, String)> = stmt
        .query_map(
            rusqlite::params![locale, version, search.unwrap(), l, (p - 1) * l],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .map(|rows| rows.filter_map(Result::ok).collect::<Vec<_>>())?;

    builder
        .body(Body::from(serde_json::to_vec(&serde_json::json!(rows))?))
        .map_err(Into::into)
}

pub async fn execute(port: u16) -> Result<()> {
    let mut root = std::env::current_dir()?;

    if !root.ends_with("search") {
        root.push("search");
    }

    tracing::info!(root = root.to_str());

    let manager = SqliteConnectionManager::file(root.join("docs.db")).with_init(move |conn| {
        // let manager = SqliteConnectionManager::memory().with_init(move |conn| {
        unsafe {
            let path = root.join("libsimple");
            tracing::trace!("{:?}", path);
            conn.load_extension(path, None)?;
        }
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

    let addr = ([0, 0, 0, 0], port).into();

    let server = Server::bind(&addr).serve(make_svc);

    tracing::info!("docse {}", addr);

    server.await.map_err(Into::into)
}
