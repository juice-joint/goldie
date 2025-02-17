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

    #[error("Video extraction processing failed: {0}")]
    VideoExtractError(String),

    #[error("Command execution failed: {0}")]
    CommandError(#[from] std::io::Error),
}

// Core video download functionality
#[derive(Clone)]
pub struct YtDownloader {
}

impl YtDownloader {

    pub async fn download(
        &self,
        yt_link: &str,
        base_dir: &str,
        file_name: &str,
    ) -> Result<(String, String, String), VideoProcessError> {
        let args = vec![
            "-f".to_string(),
            "bestvideo[height<=720][vcodec^=avc1]+bestaudio".to_string(),
            "-o".to_string(),
            format!("{}/{}/{}.%(ext)s", base_dir, file_name, file_name),
            "--merge-output-format".to_string(),
            "mp4".to_string(),
            "--restrict-filenames".to_string(),
            "--get-filename".to_string(),
            "--no-simulate".to_string(),
            "--".to_string(),
            format!("{}", yt_link.to_string()),
        ];

        //println!("yt-dlp command: {:?}", args);

        let output = Command::new("yt-dlp")
            .args(&args)
            .output()
            .map_err(VideoProcessError::CommandError)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VideoProcessError::DownloadError(stderr.to_string()));
        }

        self.parse_filename_and_extension(&output.stdout)
    }

    fn parse_filename_and_extension(&self, output: &[u8]) -> Result<(String, String, String), VideoProcessError> {
        let filename = String::from_utf8(output.to_vec())
            .map_err(|e| VideoProcessError::FilenameError(e.to_string()))?
            .trim()
            .to_string();
    
        // Split the path into components
        let path_parts: Vec<&str> = filename.rsplitn(2, '/').collect();
        if path_parts.len() != 2 {
            return Err(VideoProcessError::FilenameError("Invalid path format".to_string()));
        }
    
        let full_filename = path_parts[0];
        let directory = path_parts[1];
    
        // Split the filename and extension
        let (name, ext) = full_filename
            .rsplit_once('.')
            .ok_or_else(|| VideoProcessError::FilenameError("Invalid filename format".to_string()))?;
    
        Ok((
            directory.to_string(),
            name.to_string(),
            ext.to_string(),
        ))
    }
}
