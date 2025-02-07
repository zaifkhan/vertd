use anyhow::anyhow;
use converter::{
    format::ConverterFormat, input::ConverterInput, output::ConverterOutput, Converter,
};
use env_logger::Env;
use log::info;
use tokio::fs;

mod converter;

const FILENAME: &str = "input.mp4";

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

    let bytes = fs::read(FILENAME).await?;

    let input = ConverterInput::new(
        ConverterFormat::from_str(FILENAME)
            .ok_or_else(|| anyhow!("failed to convert filename to string"))?,
        bytes,
    );
    let output = ConverterOutput::new(ConverterFormat::WebM);

    let converter = Converter::new(input, output);
    let (job, mut channel) = converter.convert().await?;
    while let Some(progress) = channel.recv().await {
        info!("progress: {:?}", progress);
    }
    Ok(())
}
