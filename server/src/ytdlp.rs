use std::path::PathBuf;

#[derive(Clone)]
pub struct Fetcher {
}

impl Fetcher {
    pub fn download_video_from_url(&self, url: String, output: impl AsRef<str>) {
        println!("downloading video...")
    }
}

#[derive(Clone)]
pub struct Ytdlp {
    pub fetcher: Fetcher
}   

#[derive(Debug)]
pub enum YtdlpError {
    SomethingWentWrong(String)
}

impl Ytdlp {
    pub async fn new() -> Result<Self, YtdlpError> {
        let executables_dir = PathBuf::from("libs");
        let output_dir = PathBuf::from("output");
        
        let fetcher = Fetcher {};

        Ok(Ytdlp { fetcher })
    }
}


