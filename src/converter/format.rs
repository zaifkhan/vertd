use std::collections::HashMap;

use futures_util::TryFutureExt;
use lazy_static::lazy_static;

use super::{gpu::ConverterGPU, speed::ConversionSpeed};

lazy_static! {
    pub static ref FORMATS: HashMap<&'static str, ConverterFormat> = {
        let mut map = HashMap::new();
        map.insert("mp4", ConverterFormat::MP4);
        map.insert("webm", ConverterFormat::WebM);
        map.insert("avi", ConverterFormat::AVI);
        map.insert("mkv", ConverterFormat::MKV);
        map
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConverterFormat {
    MP4,
    WebM,
    AVI,
    MKV,
}

impl ConverterFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        let ext = s.split('.').last().or_else(|| Some(s))?;
        FORMATS.get(ext).copied()
    }

    pub fn to_str(&self) -> &str {
        FORMATS.iter().find(|(_, v)| **v == *self).unwrap().0
    }

    pub fn conversion_into_args(
        &self,
        speed: &ConversionSpeed,
        gpu: Option<&ConverterGPU>,
        bitrate: u64,
    ) -> Vec<String> {
        speed.to_args(self, gpu, bitrate)
    }
}

pub struct Conversion {
    pub from: ConverterFormat,
    pub to: ConverterFormat,
}

impl Conversion {
    pub fn new(from: ConverterFormat, to: ConverterFormat) -> Self {
        Self { from, to }
    }

    async fn accelerated_or_default_codec(
        &self,
        gpu: Option<&ConverterGPU>,
        codecs: &[&str],
        default: &str,
    ) -> String {
        for codec in codecs {
            if let Some(gpu) = gpu {
                if let Ok(encoder) = gpu.get_accelerated_codec(codec).await {
                    return encoder;
                }
            }
        }
        default.to_string()
    }

    pub async fn to_args(
        &self,
        speed: &ConversionSpeed,
        gpu: Option<&ConverterGPU>,
        bitrate: u64,
    ) -> anyhow::Result<Vec<String>> {
        let conversion_opts: Vec<String> = match self.to {
            ConverterFormat::MP4 | ConverterFormat::MKV => {
                let encoder = self
                    .accelerated_or_default_codec(gpu, &["h264"], "libx264")
                    .await;
                vec![
                    "-c:v".to_string(),
                    encoder,
                    "-c:a".to_string(),
                    "aac".to_string(),
                    "-strict".to_string(),
                    "experimental".to_string(),
                ]
            }
            ConverterFormat::WebM => {
                let encoder = self
                    .accelerated_or_default_codec(gpu, &["vp8", "vp9"], "libvpx")
                    .await;
                vec![
                    "-c:v".to_string(),
                    encoder.to_string(),
                    "-c:a".to_string(),
                    "libvorbis".to_string(),
                ]
            }
            ConverterFormat::AVI => vec![
                "-c:v".to_string(),
                "mpeg4".to_string(),
                "-c:a".to_string(),
                "libmp3lame".to_string(),
            ],
        };

        let conversion_opts = conversion_opts
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let result = [
            conversion_opts,
            self.to.conversion_into_args(speed, gpu, bitrate),
        ]
        .concat();

        Ok(result)
    }
}
