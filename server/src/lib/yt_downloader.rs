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
    base_dir: String,
}

impl YtDownloader {
    pub fn new(base_dir: String) -> Self {
        Self { base_dir }
    }

    pub async fn download(
        &self,
        yt_link: &str,
        file_path: &str,
    ) -> Result<(String, String), VideoProcessError> {
        let args = vec![
            "-f".to_string(),
            "bestvideo[height<=720][vcodec^=avc1]+bestaudio".to_string(),
            "-o".to_string(),
            format!("{}/{}/{}.%(ext)s", self.base_dir, file_path, file_path),
            "--merge-output-format".to_string(),
            "mp4".to_string(),
            "--restrict-filenames".to_string(),
            "--get-filename".to_string(),
            "--no-simulate".to_string(),
            format!("ytsearch:{}", yt_link.to_string()),
        ];

        println!("yt-dlp command: {:?}", args);

        let output = Command::new("yt-dlp")
            .args(&args)
            .output()
            .map_err(|e| VideoProcessError::CommandError(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VideoProcessError::DownloadError(stderr.to_string()));
        }

        self.parse_filename_and_extension(&output.stdout)
    }

    fn parse_filename_and_extension(&self, output: &[u8]) -> Result<(String, String), VideoProcessError> {
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
