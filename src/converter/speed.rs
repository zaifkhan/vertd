use serde::{Deserialize, Serialize};

use super::format::ConverterFormat;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConversionSpeed {
    UltraFast,
    Fast,
    Medium,
    Slow,
    Slower,
    VerySlow,
}

impl ConversionSpeed {
    pub fn to_args(&self, to: &ConverterFormat) -> Vec<String> {
        let mut args = Vec::new();

        match to {
            ConverterFormat::MP4 | ConverterFormat::MKV => {
                args.push("-preset".to_string());
                match self {
                    ConversionSpeed::UltraFast => args.push("ultrafast".to_string()),
                    ConversionSpeed::Fast => args.push("fast".to_string()),
                    ConversionSpeed::Medium => args.push("medium".to_string()),
                    ConversionSpeed::Slow => args.push("slow".to_string()),
                    ConversionSpeed::Slower => args.push("slower".to_string()),
                    ConversionSpeed::VerySlow => args.push("veryslow".to_string()),
                };
            }

            ConverterFormat::WebM | ConverterFormat::AVI => {
                args.push("-speed".to_string());
                match self {
                    ConversionSpeed::UltraFast => args.push("4".to_string()),
                    ConversionSpeed::Fast => args.push("3".to_string()),
                    ConversionSpeed::Medium => args.push("2".to_string()),
                    ConversionSpeed::Slow => args.push("1".to_string()),
                    ConversionSpeed::Slower => args.push("0".to_string()),
                    ConversionSpeed::VerySlow => args.push("-1".to_string()),
                };
            }
        };

        args
    }
}
