use crate::db;
use chrono::prelude::*;
use chrono::DateTime;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::net::Ipv4Addr;
use warp::Filter;

#[derive(Debug, Deserialize)]
struct RssQuery {
    query_type: String,
}

#[derive(Debug, Deserialize)]
struct PageQuery {
    path: String,
}

#[derive(Debug, Deserialize)]
struct Mark {}

#[derive(Debug, Deserialize)]
struct MarkRemove {
    link: String,
}

fn ensure_path(path: &String) -> Result<String, &'static str> {
    let cleaned_path = path_clean::clean(path);
    if !(cleaned_path.starts_with("pages/")) {
        return Err("Invalid path");
    }
    Ok(cleaned_path)
}

fn page_query(query: &PageQuery) -> Result<warp::reply::Json, &'static str> {
    let page = db::query_page_link(&query.path);
    if page.is_none() {
        return Ok(warp::reply::json(&(String::from("NoPage"), String::new())));
    }
    let path = ensure_path(&format!(
        "./pages/{}.html",
        page.as_ref().unwrap().title.clone()
    ))?;
    let data = fs::read_to_string(&path).unwrap();
    let p = page.unwrap();
    let time = p.publish_datetime.clone();
    let (title, source) = {
        if !p.readed {
            db::update_page_read(&p.link).map_err(|_op| "update error")?;
        }
        (p.title.clone(), p.source.clone())
    };

    return Ok(warp::reply::json(&(
        title,
        data,
        query.path.clone(),
        time,
        source,
    )));
}

fn rss_query(query: &RssQuery) -> Result<String, Box<dyn Error>> {
    let limits = if query.query_type == "unread" {
        vec![("readed", "0")]
    } else {
        vec![]
    };
    let mut pages = db::query_pages(&limits);
    pages.sort_by(|a, b| {
        b.publish_datetime
            .parse::<DateTime<Local>>()
            .unwrap()
            .partial_cmp(&a.publish_datetime.parse::<DateTime<Local>>().unwrap())
            .unwrap()
    });

    let page_limit = if query.query_type == "unread" {
        30
    } else {
        100
    };
    let max_len = usize::min(page_limit as usize, pages.len());
    let res: Vec<String> = pages[..max_len]
        .iter()
        .map(|page| {
            let class = if page.readed { "visited" } else { "" };
            let max = 65;
            let title = if page.title.chars().count() > max {
                page.title.chars().take(max).collect::<String>() + "..."
            } else {
                page.title.clone()
            };
            format!(
                "<li><a class=\"{}\" id=\"{}\", href=\"#\">{}</a></li>",
                class, page.link, title
            )
        })
        .collect();
    Ok(res.join(""))
}

fn rss_mark(_query: &Mark) -> Result<(), Box<dyn Error>> {
    db::mark_pages_read(15)?;
    Ok(())
}

fn rss_remove(query: &MarkRemove) -> Result<(), Box<dyn Error>> {
    let link = &query.link;
    eprintln!("remove page {}", link);
    db::remove_pages_from_link(link)?;
    Ok(())
}

pub async fn run_server(port: u16) {
    pretty_env_logger::init();

    //let pages = warp::path("static").and(warp::fs::dir("./static/"));
    let routes = warp::path!("read").and(warp::fs::file("./front/public/index.html"));
    let front = warp::path("front").and(warp::fs::dir("./front/public/"));
    let routes = routes.or(front);

    let images = warp::path("static")
        .and(warp::path("images"))
        .and(warp::get())
        .and(warp::fs::dir("./ob/Pics"));

    let page_images = warp::path("pages")
        .and(warp::path("images"))
        .and(warp::fs::dir("./pages/images"));
    let routes = routes.or(images).or(page_images);

    let page = warp::path!("api" / "page")
        .and(warp::get())
        .and(warp::query::<PageQuery>())
        .map(|query: PageQuery| {
            let res = page_query(&query);
            res.unwrap()
        });
    let routes = routes.or(page);

    let rss = warp::path!("api" / "rss")
        .and(warp::get())
        .and(warp::query::<RssQuery>())
        .map(|query: RssQuery| {
            let res = rss_query(&query);
            if res.is_ok() {
                format!("{}", res.unwrap())
            } else {
                format!("no-page")
            }
        });

    let rss_mark = warp::path!("api" / "rss_mark")
        .and(warp::post())
        .and(warp::query::<Mark>())
        .map(|query: Mark| {
            let res = rss_mark(&query);
            if res.is_ok() {
                format!("ok")
            } else {
                format!("no-page")
            }
        });
    let routes = routes.or(rss).or(rss_mark);

    let rss_remove = warp::path!("api" / "rss_remove")
        .and(warp::post())
        .and(warp::body::json())
        .map(|query: MarkRemove| {
            let res = rss_remove(&query);
            if res.is_ok() {
                format!("ok")
            } else {
                format!("")
            }
        });
    let routes = routes.or(rss_remove);

    let log = warp::log("api");
    let routes = routes.with(log);
    println!("listen to : {} ...", port);

    warp::serve(routes).run((Ipv4Addr::UNSPECIFIED, port)).await
}
