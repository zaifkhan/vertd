use super::{gpu::ConverterGPU, speed::ConversionSpeed};
use log::info;
use strum_macros::{Display, EnumString};

#[derive(Clone, Copy, Debug, PartialEq, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
pub enum ConverterFormat {
    MP4,
    WebM,
    GIF,
    AVI,
    MKV,
    WMV,
    MOV,
    MTS,
    TS,
    M2TS,
}

impl ConverterFormat {
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
            ConverterFormat::MP4
            | ConverterFormat::MKV
            | ConverterFormat::MOV
            | ConverterFormat::MTS
            | ConverterFormat::TS
            | ConverterFormat::M2TS => {
                let encoder = self
                    .accelerated_or_default_codec(gpu, &["h264"], "libx264")
                    .await;

                let mut opts = vec![
                    "-c:v".to_string(),
                    encoder.clone(), // Clone to check its name below
                    "-c:a".to_string(),
                    "aac".to_string(),
                    "-strict".to_string(),
                    "experimental".to_string(),
                ];

                // UNIVERSAL FIX: If we're using any hardware encoder for these common containers,
                // force the output pixel format to the universally compatible yuv420p.
                // This prevents the green/pink hue artifact on all affected formats.
                if encoder.contains("vaapi") || encoder.contains("nvenc") || encoder.contains("qsv") {
                    info!("Hardware H.264 encode detected. Applying yuv420p pixel format compatibility fix.");
                    opts.extend(vec!["-pix_fmt".to_string(), "yuv420p".to_string()]);
                }

                opts
            }

            ConverterFormat::GIF => {
                vec![
                    "-filter_complex".to_string(),
                    format!(
                     "fps={},scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer",
                     fps.min(24)
                    ),
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
            ConverterFormat::AVI => {
                let encoder = self
                    .accelerated_or_default_codec(gpu, &["mpeg4"], "mpeg4")
                    .await;
                vec![
                    "-c:v".to_string(),
                    encoder,
                    "-c:a".to_string(),
                    "libmp3lame".to_string(),
                ]
            }
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