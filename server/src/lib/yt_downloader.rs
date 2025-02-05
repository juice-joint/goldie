use std::process::Command;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VideoProcessError {
    #[error("YouTube download failed: {0}")]
    DownloadError(String),
    
    #[error("Failed to process filename: {0}")]
    FilenameError(String),
    
    #[error("Pitch shift processing failed: {0}")]
    PitchShiftError(String),
    
    #[error("Command execution failed: {0}")]
    CommandError(#[from] std::io::Error),
}

// Core video download functionality
#[derive(Clone)]
pub struct YtDownloader {
    output_dir: String,
}

impl YtDownloader {
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }

    pub async fn download(&self, yt_link: &str) -> Result<(String, String), VideoProcessError> {
        let args = self.build_download_args(yt_link);
        let output = Command::new("yt-dlp")
            .args(&args)
            .output()
            .map_err(|e| VideoProcessError::CommandError(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VideoProcessError::DownloadError(stderr.to_string()));
        }

        self.parse_filename(&output.stdout)
    }

    fn build_download_args(&self, yt_link: &str) -> Vec<String> {
        vec![
            "-f".to_string(),
            "bestvideo[height<=720][vcodec^=avc1]+bestaudio".to_string(),
            "-o".to_string(),
            format!("{}/%(title)s/%(title)s.%(ext)s", self.output_dir),
            "--merge-output-format".to_string(),
            "mp4".to_string(),
            "--restrict-filenames".to_string(),
            "--get-filename".to_string(),
            "--no-simulate".to_string(),
            yt_link.to_string(),
        ]
    }

    fn parse_filename(&self, output: &[u8]) -> Result<(String, String), VideoProcessError> {
        let filename = String::from_utf8(output.to_vec())
            .map_err(|e| VideoProcessError::FilenameError(e.to_string()))?
            .trim()
            .to_string();
        
        filename
            .rsplit_once('.')
            .map(|(name, ext)| (name.to_string(), ext.to_string()))
            .ok_or_else(|| VideoProcessError::FilenameError("Invalid filename format".to_string()))
    }
}
