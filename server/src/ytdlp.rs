use std::path::PathBuf;

use yt_dlp::Youtube;

#[derive(Clone)]
pub struct Ytdlp {
    pub fetcher: Youtube
}   

#[derive(Debug)]
pub enum YtdlpError {
    SomethingWentWrong(String)
}

impl Ytdlp {
    pub async fn new() -> Result<Self, YtdlpError> {
        let executables_dir = PathBuf::from("libs");
        let output_dir = PathBuf::from("output");
        
        let fetcher = Youtube::with_new_binaries(executables_dir, output_dir)
            .await
            .map_err(|error| {
                eprintln!("error downloading video: {}", error);
                YtdlpError::SomethingWentWrong(error.to_string())
            })?;
        
        fetcher.update_downloader()
            .await
            .map_err(|error| {
                eprintln!("error downloading video: {}", error);
                YtdlpError::SomethingWentWrong(error.to_string())
            })?;

        Ok(Ytdlp { fetcher })
    }
}


