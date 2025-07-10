use serde::{Deserialize, Serialize};
use tokio::process::Command;
use uuid::Uuid;

const DEFAULT_BITRATE: u64 = 4 * 1_000_000;
const BITRATE_MULTIPLIER: f64 = 2.5;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: Uuid,
    pub auth: String,
    pub from: String,
    pub to: Option<String>,
    pub completed: bool,
    total_frames: Option<u64>,
    bitrate: Option<u64>,
    fps: Option<u32>,
}

impl Job {
    pub fn new(auth_token: String, from: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            auth: auth_token,
            from,
            to: None,
            completed: false,
            total_frames: None,
            bitrate: None,
            fps: None,
        }
    }

    // TODO: scale based on resolution
    pub async fn bitrate(&mut self) -> anyhow::Result<u64> {
        // Ok(DEFAULT_BITRATE)
        if let Some(bitrate) = self.bitrate {
            return Ok(bitrate);
        }

        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=bit_rate",
                "-of",
                "default=nokey=1:noprint_wrappers=1",
                &format!("input/{}.{}", self.id, self.from),
            ])
            .output()
            .await?;

        let bitrate = String::from_utf8(output.stdout)?;
        let bitrate = match bitrate.trim().parse::<u64>() {
            Ok(bitrate) => bitrate,
            Err(_) => DEFAULT_BITRATE,
        };

        self.bitrate = Some(bitrate);
        Ok(((bitrate as f64) * BITRATE_MULTIPLIER) as u64)
    }

    pub async fn total_frames(&mut self) -> anyhow::Result<u64> {
        if let Some(total_frames) = self.total_frames {
            return Ok(total_frames);
        }

        let path = format!("input/{}.{}", self.id, self.from);

        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-count_frames", // Changed from -count_packets for better accuracy
                "-show_entries",
                "stream=nb_read_frames", // Changed from nb_read_packets
                "-of",
                "csv=p=0",
                &path,
            ])
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("ffprobe failed to count frames: {}", stderr));
        }

        let total_frames_str = String::from_utf8(output.stdout)
            .map_err(|e| anyhow::anyhow!("failed to parse total frames from ffprobe stdout: {}", e))?;
            
        let total_frames = total_frames_str
            .trim()
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Could not parse '{}' as total frames", total_frames_str))?;


        self.total_frames = Some(total_frames);
        Ok(total_frames)
    }

    pub async fn fps(&mut self) -> anyhow::Result<u32> {
        if let Some(fps) = self.fps {
            return Ok(fps);
        }

        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=r_frame_rate",
                "-of",
                "default=nokey=1:noprint_wrappers=1",
                &format!("input/{}.{}", self.id, self.from),
            ])
            .output()
            .await?;

        // its  gonna look like "30000/1001"
        let fps = String::from_utf8(output.stdout)
            .map_err(|e| anyhow::anyhow!("failed to parse fps: {}", e))?;

        let fps_parts: Vec<&str> = fps.trim().split('/').collect();
        let fps = if fps_parts.len() == 1 {
            fps_parts[0].parse::<u32>()?
        } else if fps_parts.len() == 2 {
            let numerator = fps_parts[0].parse::<f64>()?;
            let denominator = fps_parts[1].parse::<f64>()?;
            if denominator == 0.0 { 0 } else { (numerator / denominator).round() as u32 }
        } else {
            return Err(anyhow::anyhow!("failed to parse fps from '{}'", fps));
        };

        self.fps = Some(fps);
        Ok(fps)
    }

    pub async fn bitrate_and_fps(&mut self) -> anyhow::Result<(u64, u32)> {
        let (bitrate, fps) = (self.bitrate().await?, self.fps().await?);
        Ok((bitrate, fps))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum ProgressUpdate {
    #[serde(rename = "frame", rename_all = "camelCase")]
    Frame(u64),
    #[serde(rename = "fps", rename_all = "camelCase")]
    FPS(f64),
    #[serde(rename = "error", rename_all = "camelCase")]
    Error(String),
}