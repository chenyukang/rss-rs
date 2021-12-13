use actix_files as fs;
use actix_web::http::{header, StatusCode};
use actix_web::{
    error, guard, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use base::rss;
use chrono::prelude::*;
use chrono::DateTime;
use serde::Deserialize;
use std::io;

#[derive(Debug, Deserialize)]
struct RssQuery {
    query_type: String, // unread or all
    limit: usize,
}

#[derive(Debug, Deserialize)]
struct Mark {
    index: usize,
}

fn rss_mark(_info: web::Json<Mark>) -> HttpResponse {
    let res = rss::mark_pages_read(15);
    if res.is_ok() {
        HttpResponse::Ok().content_type("text/plain").body("")
    } else {
        HttpResponse::NotFound().body("error")
    }
}

/// 404 handler
async fn p404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

async fn rss_list(info: web::Json<RssQuery>) -> HttpResponse {
    let limits = if info.query_type == "unread" {
        vec![("readed", "0")]
    } else {
        vec![]
    };
    let mut pages = rss::query_pages(&limits);
    pages.sort_by(|a, b| {
        b.publish_datetime
            .parse::<DateTime<Local>>()
            .unwrap()
            .partial_cmp(&a.publish_datetime.parse::<DateTime<Local>>().unwrap())
            .unwrap()
    });

    let page_limit = if info.query_type == "unread" { 15 } else { 100 };
    let max_len = usize::min(page_limit as usize, pages.len());
    let res: Vec<String> = pages[..max_len]
        .iter()
        .map(|page| {
            let class = if page.readed { "visited" } else { "" };
            format!(
                "<li><a class=\"{}\" id=\"{}\", href=\"#\">{}</a></li>",
                class, page.link, page.title
            )
        })
        .collect();
    HttpResponse::Ok().body(res.join(""))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // cookie session middleware
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register favicon
            // with path parameters
            .service(web::resource("/mark").route(web::post().to(rss_mark)))
            // async response body
            .service(web::resource("/error").to(|| async {
                error::InternalError::new(
                    io::Error::new(io::ErrorKind::Other, "internal error"),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }))
            .service(web::resource("/rss").route(web::get().to(rss_list)))
            // static files
            .service(fs::Files::new("/static", "static"))
            // redirect
            .service(web::resource("/").route(web::get().to(|req: HttpRequest| {
                println!("{:?}", req);
                HttpResponse::Found()
                    .header(header::LOCATION, "static/index.html")
                    .finish()
            })))
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
