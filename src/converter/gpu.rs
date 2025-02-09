use anyhow::anyhow;
use std::fmt::{self, Display, Formatter};
use wgpu::Instance;

pub enum ConverterGPU {
    AMD,
    Intel,
    NVIDIA,
    Apple, // DAMN YOU M* CHIPS
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
    match info.vendor {
        0x10DE => Ok(ConverterGPU::NVIDIA),
        0x1022 => Ok(ConverterGPU::AMD),
        0x8086 => Ok(ConverterGPU::Intel), // fun fact: intel's vendor id is 0x8086, presumably in reference to the intel 8086 processor
        0x106B => Ok(ConverterGPU::Apple),
        _ => Err(anyhow!("unknown GPU vendor: 0x{:X}", info.vendor)),
    }
}
