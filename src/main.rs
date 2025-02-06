use converter::Converter;
use env_logger::Env;
use log::{error, info};

mod converter;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("vertd")).init();
    info!("starting vertd");

    let converter = Converter::new("input.mp4", "output.webm");

    let result = converter.convert();
}
