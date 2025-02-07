use std::collections::HashMap;

use lazy_static::lazy_static;

use super::{input::ConverterInput, output::ConverterOutput};

lazy_static! {
    static ref FORMATS: HashMap<&'static str, ConverterFormat> = {
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

    pub fn conversion_into_args(&self) -> Vec<String> {
        // &["-f", self.to_str().to_string().as_str()]
        vec!["-f".to_string(), self.to_str().to_string()]
    }
}

pub struct Conversion {
    pub from: ConverterInput,
    pub to: ConverterOutput,
}

impl Conversion {
    pub fn new(input: ConverterInput, output: ConverterOutput) -> Self {
        Self {
            from: input,
            to: output,
        }
    }

    pub fn to_args(&self) -> Vec<String> {
        let conversion_opts: &[&str] = match self.to.format {
            // ConverterFormat::MP4 | ConverterFormat::MKV => &[
            //     "-c:v",
            //     "libx264",
            //     "-c:a",
            //     "aac",
            //     "-strict",
            //     "experimental",
            //     "-preset",
            //     "ultrafast",
            // ],
            // ConverterFormat::WebM => &["-c:v", "libvpx", "-c:a", "libvorbis", "-speed", "4"],
            // ConverterFormat::AVI => &["-c:v", "mpeg4", "-c:a", "libmp3lame", "-speed", "4"],

            // the above but optimized for gpu acceleration
            ConverterFormat::MP4 | ConverterFormat::MKV => &[
                "-c:v",
                "h264_nvenc",
                "-c:a",
                "aac",
                "-strict",
                "experimental",
                "-preset",
                "ultrafast",
            ],

            ConverterFormat::WebM => &["-c:v", "libvpx", "-c:a", "libvorbis", "-speed", "4"],
            ConverterFormat::AVI => &["-c:v", "mpeg4", "-c:a", "libmp3lame", "-speed", "4"],
        };

        let conversion_opts = conversion_opts
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        [conversion_opts, self.to.format.conversion_into_args()].concat()
    }
}
