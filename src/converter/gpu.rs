use anyhow::anyhow;
use log::warn;
use std::fmt::{self, Display, Formatter};
use tokio::process::Command;
use wgpu::Instance;
use std::env::consts;

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
            ConverterGPU::AMD => {
                if consts::OS == "linux" {
                    vec!["vaapi"]
                } else {
                    vec!["amf"]
                }
            },

            ConverterGPU::Intel => {
                if consts::OS == "linux" {
                    vec!["vaapi"]
                } else {
                    vec!["qsv"]
                }
            },

            ConverterGPU::NVIDIA => vec!["nvenc"],
            ConverterGPU::Apple => vec!["videotoolbox"],
        }
    }

    pub fn hwaccel_args(&self) -> &[&str] {
        match self {
            ConverterGPU::AMD => {
                if consts::OS == "linux" {
                    &["-hwaccel", "vaapi", "-hwaccel_output_format", "vaapi"]
                } else {
                    &["-hwaccel", "amf"]
                }
            },

            ConverterGPU::Intel => {
                if consts::OS == "linux" {
                    &["-hwaccel", "vaapi", "-hwaccel_output_format", "vaapi"]
                } else {
                    &["-hwaccel", "qsv"]
                }
            },

            ConverterGPU::NVIDIA => &["-hwaccel", "cuda"],
            ConverterGPU::Apple => &["-hwaccel", "videotoolbox"],
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

async fn is_docker() -> bool {
    let dockerenv = tokio::fs::metadata("/.dockerenv").await.is_ok();
    let cgroup = tokio::fs::metadata("/proc/1/cgroup").await.is_ok();
    dockerenv || cgroup
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
        0x1002 => Ok(ConverterGPU::AMD),
        0x8086 => Ok(ConverterGPU::Intel), // fun fact: intel's vendor id is 0x8086, presumably in reference to the intel 8086 processor
        0x106B | 0x0 => Ok(ConverterGPU::Apple),
        0x10000..=0x10007 if is_docker().await => {
            // https://registry.khronos.org/vulkan/specs/latest/man/html/VkVendorId.html
            // https://forums.developer.nvidia.com/t/wsl2-ubuntu-uses-llvmpipe-instead-of-nvidia-gpu-3090/319022
            warn!("*******");
            warn!("you're running vertd on a docker container, but no GPU was detected.");
            warn!("this usually is because you're running Docker under WSL or because");
            warn!("you are not passing the GPU device correctly.");
            warn!("");
            warn!("if this doesn't seem right, make sure to provide the following info when");
            warn!("asking for help:");
            warn!("- adapter name: {}", info.name);
            warn!("- adapter vendor: 0x{:X}", info.vendor);
            warn!("- backend: {}", info.backend.to_str());
            warn!("- device ID: {}", info.device);
            warn!("- device type: {:#?}", info.device_type);
            warn!("- driver: {}", info.driver);
            warn!("- driver info: {}", info.driver_info);
            warn!("");
            warn!("vertd will assume you have a NVIDIA GPU. if this isn't the case,");
            warn!("conversions will likely fail.");
            warn!("*******");

            Ok(ConverterGPU::NVIDIA)
        }
        _ => Err(anyhow!("unknown GPU vendor: 0x{:X}", info.vendor)),
    }
}