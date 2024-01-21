use crate::conf::*;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// An item within a feed
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub title: String,
    pub publish_datetime: String,
    pub link: String,
    pub source: String,
    pub website: String,
    pub readed: bool,
}

pub(crate) fn init_db(db_name: Option<&str>) -> rusqlite::Result<()> {
    let name = db_name.unwrap_or(PAGES_DB);
    if !Path::new(name).exists() {
        let conn = Connection::open(name)?;
        conn.execute_batch(
            r#"
            BEGIN;
            CREATE TABLE pages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title String NOT NULL,
                link String NOT NULL,
                website String,
                publish_datetime String,
                readed Boolean,
                source String NOT NULL);
            CREATE UNIQUE INDEX idx_pages_link ON pages (link);
            COMMIT;
            "#,
        )?;
        eprintln!("db created: {:?}", name);
    }
    Ok(())
}

pub(crate) fn all_feeds() -> Vec<String> {
    let rss_buf = fs::read_to_string(ALL_FEEDS).unwrap();
    let feeds = rss_buf
        .split("\n")
        .map(|l| l.trim())
        .filter(|&l| l.len() > 0)
        .map(|l| l.to_string())
        .collect::<Vec<_>>();
    feeds
}

pub(crate) fn cleanup_pages() -> rusqlite::Result<()> {
    let feeds = all_feeds();
    let conn = Connection::open(PAGES_DB)?;
    let params = feeds
        .iter()
        .map(|f| format!("'{}'", f))
        .collect::<Vec<String>>();
    let sql = format!(
        "DELETE FROM pages WHERE source NOT IN ({})",
        params.join(", ")
    );
    conn.execute(&sql, [])?;
    Ok(())
}

pub(crate) fn dump_new_page(page: &Page) -> rusqlite::Result<()> {
    let conn = Connection::open(PAGES_DB)?;
    if let Some(_) = query_page_link(&page.link) {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO pages (title, link, website, publish_datetime, readed, source) values (?1, ?2, ?3, ?4, ?5, ?6)",
        params![page.title, page.link, page.website, page.publish_datetime, page.readed, page.source])?;
    Ok(())
}

pub fn update_page_read(link: &str) -> rusqlite::Result<usize> {
    let conn = Connection::open(PAGES_DB)?;
    conn.execute("UPDATE pages set readed = 1 where link = ?", [link])
}

pub fn mark_pages_read(limit: usize) -> rusqlite::Result<usize> {
    let conn = Connection::open(PAGES_DB)?;
    let sql = format!(
        "UPDATE pages SET readed = 1 WHERE id IN (SELECT id FROM pages WHERE readed = 0 ORDER BY publish_datetime DESC LIMIT {})",
        limit
    );
    let res = conn.execute(&sql, []);
    println!("result: {:?}", res);
    res
}

pub fn remove_pages_from_link(link: &str) -> rusqlite::Result<usize> {
    let conn = Connection::open(PAGES_DB)?;
    let Some(page) = query_page_link(&link) else {
        return Ok(0);
    };
    let res = conn.execute("DELETE FROM pages where source = ?", [&page.source]);
    // remove feed from feeds.md
    let feeds = all_feeds();
    let feeds = feeds
        .iter()
        .filter(|&f| f != &page.source)
        .map(|f| f.to_string())
        .collect::<Vec<_>>();
    fs::write(ALL_FEEDS, feeds.join("\n")).unwrap();
    eprintln!("deleted {:#?}", res);
    res
}

pub fn query_pages(limits: &Vec<(&str, &str)>) -> Vec<Page> {
    #[cfg(not(test))]
    cleanup_pages().unwrap();
    let conn = Connection::open(PAGES_DB).unwrap();
    let limit_str = if limits.len() > 0 {
        limits
            .iter()
            .map(|&(k, v)| format!("{} = '{}'", k, v))
            .collect::<Vec<String>>()
            .join(" AND ")
    } else {
        String::from(" 1 = 1 ")
    };
    let sql = format!(
        "SELECT * FROM pages WHERE {} ORDER BY publish_datetime DESC",
        limit_str
    );
    let mut statement = conn.prepare(&sql).unwrap();
    let pages = statement
        .query_map([], |row| {
            Ok(Page {
                title: row.get(1).unwrap_or("no title".to_string()),
                link: row.get(2).unwrap(),
                website: row.get(3).unwrap(),
                publish_datetime: row.get(4).unwrap(),
                readed: row.get(5).unwrap(),
                source: row.get(6).unwrap(),
            })
        })
        .unwrap();

    let res: Vec<Page> = pages.map(|f| f.unwrap()).collect();
    return res;
}

pub fn query_page_link(link: &str) -> Option<Page> {
    let pages = query_pages(&vec![("link", link)]);
    assert!(pages.len() <= 1);
    if pages.len() == 1 {
        return Some(pages[0].clone());
    } else {
        None
    }
}
