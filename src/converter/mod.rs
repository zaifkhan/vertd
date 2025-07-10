use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Context};
use format::{Conversion, ConverterFormat};
use job::{Job, ProgressUpdate};
use log::error;
use log::info;
use speed::ConversionSpeed;
use tokio::fs;
use tokio::io::AsyncBufReadExt as _;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::mpsc;

pub mod format;
pub mod gpu;
pub mod job;
pub mod speed;

/// Finds the first available VA-API render device.
/// e.g., /dev/dri/renderD128
async fn find_vaapi_device() -> anyhow::Result<String> {
    const DRM_DIR: &str = "/dev/dri";
    let mut entries = fs::read_dir(DRM_DIR)
        .await
        .context("Failed to read /dev/dri directory. Is a GPU driver loaded?")?;

    while let Some(entry) = entries.next_entry().await? {
        if let Some(name) = entry.file_name().to_str() {
            if name.starts_with("renderD") {
                let device_path = format!("{}/{}", DRM_DIR, name);
                info!("Found VA-API render device: {}", &device_path);
                return Ok(device_path);
            }
        }
    }

    Err(anyhow!(
        "No VA-API render device found in /dev/dri. Ensure you have passed the device to your container and drivers are installed."
    ))
}

pub struct Converter {
    pub conversion: Conversion,
    speed: ConversionSpeed,
}

impl Converter {
    pub fn new(from: ConverterFormat, to: ConverterFormat, speed: ConversionSpeed) -> Self {
        Self {
            conversion: Conversion::new(from, to),
            speed,
        }
    }

    pub async fn convert(&self, job: &mut Job) -> anyhow::Result<mpsc::Receiver<ProgressUpdate>> {
        let (tx, rx) = mpsc::channel(1);
        let input_filename = format!("input/{}.{}", job.id, self.conversion.from.to_string());
        let output_filename = format!("output/{}.{}", job.id, self.conversion.to.to_string());

        let (gpu, (bitrate, fps)) = tokio::try_join!(gpu::get_gpu(), job.bitrate_and_fps())?;

        // Determine the encoder arguments first to see if we're using hardware.
        let conversion_args = self
            .conversion
            .to_args(&self.speed, &gpu, bitrate, fps)
            .await?;

        let encoder_is_hardware = conversion_args
            .iter()
            .any(|s| s.contains("vaapi") || s.contains("nvenc") || s.contains("qsv"));

        let mut final_command = vec![
            "-hide_banner".to_string(),
            "-loglevel".to_string(),
            "error".to_string(),
            "-progress".to_string(),
            "pipe:1".to_string(),
        ];

        // If using a hardware encoder, we must initialize the device context *before* the input.
        if encoder_is_hardware {
            info!("Hardware encoder detected, preparing for CPU->GPU frame upload.");
            let device = find_vaapi_device().await?;
            final_command.extend_from_slice(&[
                "-init_hw_device".to_string(),
                format!("vaapi=hwdevice:{}", device), // Create a device named "hwdevice"
                "-filter_hw_device".to_string(),
                "hwdevice".to_string(), // Tell filters to use it
            ]);
        }

        // Add input file
        final_command.extend_from_slice(&["-i".to_string(), input_filename.to_string()]);

        // Add filters if needed
        if encoder_is_hardware {
            // This is a more robust filter chain. It uploads the frame, then uses the GPU's
            // own scaler to ensure the frame is in the NV12 format required by the encoder.
            final_command.extend_from_slice(&[
                "-vf".to_string(),
                "hwupload,scale_vaapi=format=nv12".to_string(),
            ]);
        }

        // Add the rest of the arguments (encoder, bitrate, etc.) and the output file
        final_command.extend(conversion_args);
        final_command.push(output_filename);

        info!("running 'ffmpeg {}'", final_command.join(" "));

        let mut process = Command::new("ffmpeg")
            .args(&final_command)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("failed to spawn ffmpeg: {}", e))?;

        let stderr = process
            .stderr
            .take()
            .ok_or_else(|| anyhow!("failed to take stderr"))?;

        let tx_arc = Arc::new(tx);

        let tx_clone = Arc::clone(&tx_arc);

        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                error!("{}", line);
                let _ = tx_clone.send(ProgressUpdate::Error(line)).await;
            }
        });

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| anyhow!("failed to take stdout"))?;
        let reader = BufReader::new(stdout);

        let tx_clone = Arc::clone(&tx_arc);

        tokio::spawn(async move {
            let mut lines = reader.lines();
            while let Ok(Some(out)) = lines.next_line().await {
                let mut map = HashMap::new();
                for line in out.split('\n') {
                    if let Some((k, v)) = line.split_once('=') {
                        map.insert(k.trim(), v.trim());
                    }
                }

                let mut reports = Vec::new();

                if let Some(frame) = map.get("frame").and_then(|s| s.parse().ok()) {
                    reports.push(ProgressUpdate::Frame(frame));
                }

                if let Some(fps) = map.get("fps").and_then(|s| s.parse().ok()) {
                    reports.push(ProgressUpdate::FPS(fps));
                }

                for report in reports {
                    if tx_clone.send(report).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}