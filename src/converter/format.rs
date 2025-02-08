use std::collections::HashMap;

use lazy_static::lazy_static;

use super::speed::ConversionSpeed;

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

    pub fn conversion_into_args(&self, speed: &ConversionSpeed) -> Vec<String> {
        speed.to_args(self)
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

    pub fn to_args(&self, speed: &ConversionSpeed) -> Vec<String> {
        let conversion_opts: &[&str] = match self.to {
            ConverterFormat::MP4 | ConverterFormat::MKV => &[
                "-c:v",
                "h264_nvenc",
                "-c:a",
                "aac",
                "-strict",
                "experimental",
            ],

            // TODO: add support for VP9
            ConverterFormat::WebM => &["-c:v", "libvpx", "-c:a", "libvorbis"],
            ConverterFormat::AVI => &["-c:v", "mpeg4", "-c:a", "libmp3lame"],
        };

        let conversion_opts = conversion_opts
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        [conversion_opts, self.to.conversion_into_args(speed)].concat()
    }
}
