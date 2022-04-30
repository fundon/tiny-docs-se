use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::{env, fs};

use comrak::nodes::{AstNode, NodeValue};
use comrak::{format_html, parse_document, Arena, ComrakOptions};
use serde_json::{Map, Value};

use rusqlite::{params, Connection};

// (gid, tag, content, (level, parent id), draft)
#[derive(Debug)]
struct Meta(Option<String>, String, String, Vec<(u32, i64)>, bool);

pub fn execute(args: &ArgMatches) -> Result<()> {
    let root = env::current_dir()?;

    let mut options = ComrakOptions::default();

    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.table = true;

    let conn = rusqlite::Connection::open("docs.db")?;

    for entry in glob::glob(root.join("**/*.md").to_str().ok_or(anyhow!("Missing"))?)? {
        let path = entry?;

        if path.starts_with(root.join(".github")) || path.starts_with(root.join("TOC.md")) {
            continue;
        }

        tracing::info!(path = path.to_str());

        let file = fs::read(path)?;
        let md = String::from_utf8_lossy(&file);

        let arena = Arena::new();
        let root = parse_document(&arena, &md, &options);

        let mut meta = Meta(None, "".to_string(), "".to_string(), Vec::new(), false);

        fn iter_nodes<'a, F>(
            node: &'a AstNode<'a>,
            f: &F,
            conn: &Connection,
            meta: &mut Meta,
        ) -> Result<()>
        where
            F: Fn(&'a AstNode<'a>, &Connection, &mut Meta) -> Result<()>,
        {
            f(node, conn, meta)?;
            for c in node.children() {
                iter_nodes(c, f, conn, meta)?
            }
            Ok(())
        }

        iter_nodes(
            root,
            &|node, conn, meta| {
                match &mut node.data.borrow_mut().value {
                    NodeValue::Document => {
                        meta.0.replace(uuid::Uuid::new_v4().to_string());
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

                        if let Some(draft) = map.get("draft").and_then(|v| v.as_bool()) {
                            meta.4 = draft;
                            if draft {
                                return Ok(());
                            }
                        }
                        if let Some(uuid) = map
                            .get("uuid")
                            .and_then(|v| v.as_str())
                            .filter(|v| !v.is_empty())
                        {
                            meta.0.replace(uuid.to_string());
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
                    n => {
                        match n {
                            NodeValue::Heading(ref mut v) => {
                                let t = format!("h{}", v.level);
                                /*
                                if !meta.1.is_empty() {
                                    if meta.1 == "p" {
                                        // dbg!(&meta.1, &meta.2, &meta.3);
                                        conn.execute("insert into docs(uuid, kind, content, parent) VALUES (?1, ?2, ?3, ?4)", params![meta.0.clone().unwrap(), meta.1, meta.2.trim_start().trim_end(), meta.3.last().unwrap().1])?;
                                        meta.2.clear();
                                    } else {
                                        let p = meta
                                            .3
                                            .iter()
                                            .rfind(|(l, _)| l < &v.level)
                                            .cloned()
                                            .unwrap_or_default();
                                        conn.execute("insert into docs(uuid, kind, content, parent) VALUES (?1, ?2, ?3, ?4)", params![meta.0.clone().unwrap(), meta.1, meta.2.trim_start().trim_end(), p.1])?;
                                        meta.3.push((v.level, conn.last_insert_rowid()));
                                        meta.2.clear();
                                    }
                                }

                                meta.1 = t;
                                */
                            }
                            NodeValue::Text(ref mut v) => {
                                let text = String::from_utf8_lossy(&v);
                                meta.2.push_str(&text);
                            }
                            NodeValue::Code(ref mut v) => {
                                let text = String::from_utf8_lossy(&v.literal);
                                meta.2.push_str(&text);
                            }
                            NodeValue::TableCell => {
                                meta.2.push_str("\n");
                            }
                            _ => {
                                if meta.1 != "p" {
                                /*
                                    let v: u32 = meta
                                        .1
                                        .chars()
                                        .last()
                                        .and_then(|v| v.to_string().parse().ok())
                                        .unwrap();
                                    let p = meta
                                        .3
                                        .iter()
                                        .rfind(|(l, _)| l < &v)
                                        .cloned()
                                        .unwrap_or_default();
                                    conn.execute("insert into docs(uuid, kind, content, parent) VALUES (?1, ?2, ?3, ?4)", params![meta.0.clone().unwrap(), meta.1, meta.2, p.1])?;
                                    meta.3.push((v, conn.last_insert_rowid()));
                                    meta.2.clear();
                                    meta.1 = "p".to_string();
                                    */
                                } else {
                                    meta.2.push_str("\n");
                                }
                            }
                        }
                    }
                }

                Ok(())
            },
            &conn,
            &mut meta,
        )?;

        if !meta.4 {
            /*
            if meta.1 == "p" {
                conn.execute(
                    "insert into docs(uuid, kind, content, parent) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        meta.0.clone().unwrap(),
                        meta.1,
                        meta.2.trim_start().trim_end(),
                        meta.3.last().unwrap().1
                    ],
                )?;
                meta.2.clear();
            } else {
                let v = meta.3.last().cloned().unwrap_or_default();
                let p = meta.3.iter().rfind(|(l, _)| l < &v.0);
                conn.execute(
                    "insert into docs(uuid, kind, content, parent) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        meta.0.clone().unwrap(),
                        meta.1,
                        meta.2.trim_start().trim_end(),
                        p.unwrap().1
                    ],
                )?;
                meta.3.push((
                    meta.1
                        .chars()
                        .last()
                        .and_then(|v| v.to_string().parse().ok())
                        .unwrap(),
                    conn.last_insert_rowid(),
                ));
                meta.2.clear();
            }
            */
        }
    }

    Ok(())
}
