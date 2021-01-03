use std::cell::Cell;
use std::convert::Infallible;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::convert::TryInto;

use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::middleware;
use actix_web::web;
use actix_web::get;
use anyhow::Context;
use anyhow::Error;
use actix_web::Result;
use async_mpd::MpdClient;
use getset::Getters;
use prometheus_exporter_base::prelude::*;
use structopt::StructOpt;
use itertools::Itertools;

mod metric;
use metric::*;

#[derive(Debug, parse_display::Display, thiserror::Error)]
pub enum ApplicationError {
    #[display("IO Error: {}")]
    Io(#[from] std::io::Error),

    #[display("MPD Error")]
    MpdError(#[from] async_mpd::Error),

    #[display("Error")]
    AnyError(#[from] anyhow::Error),
}


#[derive(Getters, StructOpt)]
#[structopt(name = "prometheus-mpd-exporter", about = "Export MPD metrics to prometheus")]
struct Opt {
    #[structopt(long)]
    #[getset(get = "pub")]
    mpd_server_addr: String,

    #[structopt(long)]
    #[getset(get = "pub")]
    mpd_server_port: u16,

    #[structopt(long)]
    #[getset(get = "pub")]
    bind_addr: String,

    #[structopt(long)]
    #[getset(get = "pub")]
    bind_port: u16,
}

#[derive(Debug, Clone, Default)]
struct PrometheusOptions {}

async fn index(mpd_data: web::Data<Mutex<MpdClient>>, req: HttpRequest) -> impl Responder {
    match index_handler(mpd_data).await {
        Ok(text) => {
            HttpResponse::build(StatusCode::OK)
                .content_type("text/text; charset=utf-8")
                .body(text)
        }

        Err(e) => {
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("text/text; charset=utf-8")
                .body(format!("{}", e))
        }
    }
}

async fn index_handler(mpd_data: web::Data<Mutex<MpdClient>>) -> Result<String, ApplicationError> {
    let mut mpd = mpd_data.lock().unwrap();
    let stats = mpd.stats().await?;

    let instance = String::new(); // TODO

    let res = vec![
        Metric::new("uptime"      , stats.uptime      , "The uptime of mpd", &instance).into_metric()?,
        Metric::new("playtime"    , stats.playtime    , "The playtime of the current playlist", &instance).into_metric()?,
        Metric::new("artists"     , stats.artists     , "The number of artists", &instance).into_metric()?,
        Metric::new("albums"      , stats.albums      , "The number of albums", &instance).into_metric()?,
        Metric::new("songs"       , stats.songs       , "The number of songs", &instance).into_metric()?,
        Metric::new("db_playtime" , stats.db_playtime , "The database playtime", &instance).into_metric()?,
        Metric::new("db_update"   , stats.db_update   , "The updates of the database", &instance).into_metric()?,
    ]
    .into_iter()
    .map(|m| {
        m.render()
    })
    .join("\n");

    log::debug!("res = {}", res);
    Ok(res)
}

#[actix_web::main]
async fn main() -> Result<(), ApplicationError> {
    let _ = env_logger::init();
    log::info!("Starting...");
    let opt = Opt::from_args();

    let prometheus_bind_addr = format!("{}:{}", opt.bind_addr, opt.bind_port);
    let mpd_connect_string = format!("{}:{}", opt.mpd_server_addr, opt.mpd_server_port);
    log::debug!("Connecting to MPD = {}", mpd_connect_string);
    let mpd = async_mpd::MpdClient::new(&*mpd_connect_string)
        .await
        .map(Mutex::new)?;

    let mpd = web::Data::new(mpd);

    HttpServer::new(move || {
        App::new()
            .app_data(mpd.clone()) // add shared state
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
    })
    .bind(prometheus_bind_addr)?
    .run()
    .await
    .map_err(ApplicationError::from)
}
