use clap::{value_parser, App};
use colored::Colorize;
use daemonize::Daemonize;
use rss_rs::api::*;
use rss_rs::feed;
use rss_rs::utils::*;
use std::fs::File;
use std::path::PathBuf;

fn start_auto_update_job(minutes: u64) {
    // start a new thread to update rss priodically
    if minutes <= 0 {
        return;
    };

    tokio::spawn(async move {
        loop {
            let res = tokio::task::spawn_blocking(|| feed::update_rss(None, false).unwrap()).await;
            match res {
                Ok(_) => eprintln!("RSS updated successfully"),
                Err(e) => eprintln!("Background task panicked: {:?}", e),
            }
            tokio::time::sleep(std::time::Duration::from_secs(60 * minutes)).await;
        }
    });
}

#[tokio::main]
async fn run_app(port: u16, minutes: u64) {
    start_auto_update_job(minutes);
    run_server(port).await;
}

fn main() {
    let matches = App::new("Rss-rs")
        .version("0.1")
        .author("yukang <moorekang@gmail.com>")
        .about("Rss-Rs Reader in Rust")
        .arg(
            clap::Arg::new("port")
                .short('p')
                .help("Listen port")
                .takes_value(true)
                .default_value("8005")
                .value_parser(value_parser!(u16)),
        )
        .arg(clap::Arg::new("daemon").short('d').help("Run as daemon"))
        .arg(
            clap::Arg::new("update")
                .short('u')
                .help("Start a background job to fetch articles from feed")
                .takes_value(true)
                .default_value("20")
                .value_parser(value_parser!(u64)),
        )
        .arg(clap::Arg::new("stop").short('s').help("Stop daemon"))
        .get_matches();

    let port = *matches.get_one::<u16>("port").unwrap();
    let minutes = *matches.get_one::<u64>("update").unwrap();
    let daemon = matches.is_present("daemon");

    let pid_file: PathBuf = format!("/tmp/rss-rs-{}.pid", port).into();
    if daemon {
        if check_process(&pid_file).is_ok() {
            eprintln!("{}", "rss-rs is already running".red());
            return;
        }

        let pwd = std::env::current_dir().unwrap();
        let log_file = File::create("/tmp/rss-rs.log").unwrap();
        let daemonize = Daemonize::new()
            .pid_file(format!("/tmp/rss-rs-{}.pid", port))
            .stdout(log_file)
            .working_directory(pwd)
            .privileged_action(|| "Executed before drop privileges");
        match daemonize.start() {
            Ok(_) => {
                println!("Success, daemonized");
                run_app(port, minutes);
            }
            Err(e) => eprintln!("Error, {}", e),
        }
    } else if matches.is_present("stop") {
        kill_process(&pid_file, "rss-rs").unwrap();
    } else {
        run_app(port, minutes);
    }
}
