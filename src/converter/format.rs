use std::collections::HashMap;

use lazy_static::lazy_static;

use super::{gpu::ConverterGPU, speed::ConversionSpeed};

lazy_static! {
    pub static ref FORMATS: HashMap<&'static str, ConverterFormat> = {
        let mut map = HashMap::new();
        map.insert("mp4", ConverterFormat::MP4);
        map.insert("webm", ConverterFormat::WebM);
        map.insert("gif", ConverterFormat::GIF);
        map.insert("avi", ConverterFormat::AVI);
        map.insert("mkv", ConverterFormat::MKV);
        map.insert("wmv", ConverterFormat::WMV);
        map.insert("mov", ConverterFormat::MOV);
        map
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConverterFormat {
    MP4,
    WebM,
    GIF,
    AVI,
    MKV,
    WMV,
    MOV,
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
        gpu: &ConverterGPU,
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
        gpu: &ConverterGPU,
        codecs: &[&str],
        default: &str,
    ) -> String {
        for codec in codecs {
            if let Ok(encoder) = gpu.get_accelerated_codec(codec).await {
                return encoder;
            }
        }
        default.to_string()
    }

    pub async fn to_args(
        &self,
        speed: &ConversionSpeed,
        gpu: &ConverterGPU,
        bitrate: u64,
        fps: u32,
    ) -> anyhow::Result<Vec<String>> {
        let conversion_opts: Vec<String> = match self.to {
            ConverterFormat::MP4 | ConverterFormat::MKV | ConverterFormat::MOV => {
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

            ConverterFormat::GIF => {
                vec![
                   "-filter_complex".to_string(), 
                   format!(
                    "fps={},scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer",
                    fps.min(24)
                   )
                ]
            }

            ConverterFormat::WMV => {
                let encoder = self
                    .accelerated_or_default_codec(gpu, &["wmv2", "wmv3"], "wmv2")
                    .await;
                vec![
                    "-c:v".to_string(),
                    encoder,
                    "-c:a".to_string(),
                    "wmav2".to_string(),
                ]
            }
            ConverterFormat::WebM => {
                let encoder = self
                    .accelerated_or_default_codec(gpu, &["av1", "vp9", "vp8"], "libvpx")
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
