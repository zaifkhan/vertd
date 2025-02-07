use env_logger::Env;
use http::start_http;
use log::info;
use tokio::{fs, task};
use ws::start_ws;

mod converter;
mod http;
mod ws;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("vertd")).init();
    info!("starting vertd");

    // remove input/ and output/ recursively if they exist -- we don't care if this fails tho
    let _ = fs::remove_dir_all("input").await;
    let _ = fs::remove_dir_all("output").await;

    // create input/ and output/ directories
    fs::create_dir("input").await?;
    fs::create_dir("output").await?;

    // start ws and http server on separate threads
    let ws_thread = task::spawn(start_ws());
    let http_thread = task::spawn(start_http());

    // everything runs on diff threads -- http server, ws server and conversion
    // this allows for max speed!!! rocket emoji
    let r = tokio::try_join!(ws_thread, http_thread)?;
    r.0?;
    r.1?;
    Ok(())
}
