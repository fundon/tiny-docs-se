use std::{env, fs, path::PathBuf};

use anyhow::{anyhow, Result};
use comrak::{
    nodes::{AstNode, NodeValue},
    {parse_document, Arena, ComrakOptions},
};
use rusqlite::params;
use serde_json::{Map, Value};

// (tag, content, gid|path)
#[derive(Debug, Clone)]
struct Meta(u32, String, String);

pub fn execute(root: PathBuf, locale: String, version: String) -> Result<()> {
    let db_path = env::current_dir()?;

    tracing::info!(
        root = root.to_str(),
        locale = locale.as_str(),
        version = version.as_str(),
    );

    let mut options = ComrakOptions::default();

    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.table = true;

    let conn = rusqlite::Connection::open(db_path.join("docs.db"))?;

    unsafe {
        conn.load_extension(db_path.join("libsimple"), None)?;
    }

    conn.execute(
        "DELETE FROM docs WHERE locale = ?1 AND version = ?2",
        params![locale, version],
    )?;
    conn.execute(
        "DELETE FROM d WHERE locale = ?1 AND version = ?2",
        params![locale, version],
    )?;

    for entry in glob::glob(root.join("**/*.md").to_str().ok_or(anyhow!("Missing"))?)? {
        let path = entry?;

        if path.starts_with(root.join(".github")) || path.ends_with(root.join("TOC.md")) {
            continue;
        }

        let gid_path = path.clone();
        let mut gid = gid_path
            .strip_prefix(root.clone())
            .ok()
            .and_then(|p| p.to_str())
            .and_then(|p| p.strip_suffix(".md"))
            .unwrap();

        gid = gid.trim_end_matches("README").trim_end_matches("/");

        tracing::info!(path = path.to_str(), gid = gid);

        let mut draft = false;
        // let gid = path.clone().to_str().unwrap().to_string();
        let file = fs::read(path)?;
        let md = String::from_utf8_lossy(&file);

        let arena = Arena::new();
        let root = parse_document(&arena, &md, &options);

        let mut metas = Vec::new();
        let mut meta = Meta(0, "".to_string(), gid.to_string());

        fn iter_nodes<'a, F>(
            node: &'a AstNode<'a>,
            f: &F,
            draft: &mut bool,
            meta: &mut Meta,
            metas: &mut Vec<Meta>,
        ) -> Result<()>
        where
            F: Fn(&'a AstNode<'a>, &mut bool, &mut Meta, &mut Vec<Meta>) -> Result<()>,
        {
            f(node, draft, meta, metas)?;
            for m in node.children() {
                iter_nodes(m, f, draft, meta, metas)?
            }
            Ok(())
        }

        iter_nodes(
            root,
            &|node, draft, meta, metas| {
                match &mut node.data.borrow_mut().value {
                    NodeValue::Document => {
                        // meta.0.replace(uuid::Uuid::new_v4().to_string());
                        // meta.0 = path.to_str().unwrap().to_string();
                    }
                    NodeValue::FrontMatter(ref mut v) => {
                        let text = String::from_utf8_lossy(&v);
                        let header = text
                            .trim_start()
                            .trim_start_matches("---")
                            .trim_end()
                            .trim_end_matches("---");

                        tracing::debug!(header = header);

                        let map = dbg!(serde_yaml::from_str::<Map<String, Value>>(&header))?;

                        if let Some(d) = map.get("draft").and_then(|v| v.as_bool()) {
                            *draft = d;
                        }
                        if let Some(title) = map
                            .get("title")
                            .and_then(|v| v.as_str())
                            .filter(|v| !v.is_empty())
                        {
                            // conn.execute("insert into docs(uuid, kind, content, parent) VALUES (?1, ?2, ?3, ?4)", params![meta.0.clone().unwrap(), "t", title, 0])?;
                        }
                        if let Some(summary) = map
                            .get("summary")
                            .and_then(|v| v.as_str())
                            .filter(|v| !v.is_empty())
                        {
                            // conn.execute("insert into docs(uuid, kind, content, parent) VALUES (?1, ?2, ?3, ?4)", params![meta.0.clone().unwrap(), "s", summary, 0])?;
                        }

                        tracing::debug!("{:?}", map);

                        v.clear();
                    }
                    NodeValue::HtmlBlock(ref mut v) => {
                        v.literal.clear();
                    }
                    n => match n {
                        NodeValue::Heading(ref mut v) => {
                            if meta.0 != 0 {
                                metas.push(meta.clone());
                                meta.1.clear();
                            }
                            meta.0 = v.level;
                        }
                        NodeValue::Text(ref mut v) => {
                            meta.1.push_str(&String::from_utf8_lossy(&v));
                        }
                        NodeValue::Code(ref mut v) => {
                            meta.1.push_str(&String::from_utf8_lossy(&v.literal));
                        }
                        NodeValue::TableCell => {
                            meta.1.push_str("\n");
                        }
                        NodeValue::Link(ref mut v) => {
                            meta.1.push_str(&String::from_utf8_lossy(&v.title));
                        }
                        NodeValue::HtmlInline(ref mut v) => {
                            if meta.0 != 7 {
                                meta.1.clear();
                            }
                        }
                        NodeValue::Paragraph => {
                            if meta.0 != 7 {
                                metas.push(meta.clone());
                                meta.0 = 7;
                                meta.1.clear();
                            }
                        }
                        _ => {
                            if meta.0 == 7 {
                                meta.1.push_str("\n");
                            } else {
                                metas.push(meta.clone());
                                meta.0 = 7;
                                meta.1.clear();
                            }
                        }
                    },
                }

                Ok(())
            },
            &mut draft,
            &mut meta,
            &mut metas,
        )?;

        metas.push(meta.clone());

        let mut pid = 0;

        if !draft {
            for Meta(tag, content, gid) in metas {
                if tag > 0 {
                    pid = conn.query_row("select id from docs where gid = ?1 and tag <= ?2 and locale = ?3 and version = ?4 order by id desc limit 1", params![
        gid, tag - 1, locale, version
                            ], |row| row.get(0)).unwrap_or_default();
                }
                conn.execute("insert into docs(pid, gid, tag, content, locale, version) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![
                            pid, gid, tag, content, locale, version
                        ])?;
            }
        }
    }

    conn.execute(
        "INSERT INTO d SELECT * FROM docs WHERE locale = ?1 AND version = ?2",
        params![locale, version],
    )?;

    Ok(())
}
