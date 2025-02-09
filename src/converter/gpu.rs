use anyhow::anyhow;
use std::fmt::{self, Display, Formatter};
use tokio::process::Command;
use wgpu::Instance;

pub enum ConverterGPU {
    AMD,
    Intel,
    NVIDIA,
    Apple,
}

impl ConverterGPU {
    pub async fn get_accelerated_codec(&self, codec: &str) -> anyhow::Result<String> {
        let priority = self.encoder_priority();
        let encoders = Command::new("ffmpeg")
            .args(["-hide_banner", "-encoders"])
            .output()
            .await
            .map_err(|e| anyhow!("failed to get encoder support: {}", e))?;
        let encoders = String::from_utf8(encoders.stdout)?;
        for encoder in priority {
            let encoder = format!("{}_{}", codec, encoder);
            if encoders.contains(&encoder) {
                return Ok(encoder);
            }
        }

        Err(anyhow!("no supported encoder found for {}", codec))
    }

    pub fn encoder_priority(&self) -> Vec<&str> {
        match self {
            ConverterGPU::AMD => vec!["amf"],
            ConverterGPU::Intel => vec!["qsv"],
            ConverterGPU::NVIDIA => vec!["nvenc"],
            ConverterGPU::Apple => vec!["videotoolbox"],
        }
    }
}

impl Display for ConverterGPU {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConverterGPU::AMD => write!(f, "AMD"),
            ConverterGPU::Intel => write!(f, "Intel"),
            ConverterGPU::NVIDIA => write!(f, "NVIDIA"),
            ConverterGPU::Apple => write!(f, "Apple"),
        }
    }
}

pub async fn get_gpu() -> anyhow::Result<ConverterGPU> {
    let instance = Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| anyhow!("no compatible adapter found"))?;

    let info = adapter.get_info();
    if info.name.contains("Apple") {
        return Ok(ConverterGPU::Apple);
    }
    match info.vendor {
        0x10DE => Ok(ConverterGPU::NVIDIA),
        0x1022 => Ok(ConverterGPU::AMD),
        0x8086 => Ok(ConverterGPU::Intel), // fun fact: intel's vendor id is 0x8086, presumably in reference to the intel 8086 processor
        0x106B | 0x0 => Ok(ConverterGPU::Apple),
        _ => Err(anyhow!("unknown GPU vendor: 0x{:X}", info.vendor)),
    }
}
