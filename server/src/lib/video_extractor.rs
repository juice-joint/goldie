use std::process::Command;
use std::path::Path;
use std::fs;

pub struct DashVideoProcessor {
    input_file: String,
    output_dir: String,
}

impl DashVideoProcessor {
    pub fn new(input_file: &str, output_dir: &str) -> Self {
        DashVideoProcessor {
            input_file: input_file.to_string(),
            output_dir: output_dir.to_string(),
        }
    }

    fn build_adaptation_sets(&self) -> String {
        "id=0,streams=0".to_string()
    }

    fn build_stream_mappings(&self) -> Vec<String> {
        vec!["-map".to_string(), "0:v".to_string()]
    }

    fn build_video_encodings(&self) -> Vec<String> {
        vec!["-c:v".to_string(), "copy".to_string()]
    }

    fn get_output_path(&self) -> String {
        format!("{}/video/stream.mpd", self.output_dir)
    }

    fn ensure_output_directories(&self) -> std::io::Result<()> {
        // Create main output directory if it doesn't exist
        if !Path::new(&self.output_dir).exists() {
            fs::create_dir_all(&self.output_dir)?;
        }

        // Create video subdirectory
        let video_dir = format!("{}/video", self.output_dir);
        if !Path::new(&video_dir).exists() {
            fs::create_dir_all(&video_dir)?;
        }

        Ok(())
    }

    pub fn execute(&self) -> std::io::Result<()> {
        // Ensure output directories exist before processing
        self.ensure_output_directories()?;

        let mut command = Command::new("ffmpeg");
        command
            .arg("-i")
            .arg(&self.input_file)
            .arg("-threads")
            .arg("16")
            // Enable more aggressive thread-based optimization
            .arg("-thread_type")
            .arg("frame")
            .args(self.build_stream_mappings())
            .args(self.build_video_encodings())
            .arg("-f")
            .arg("dash")
            .arg("-adaptation_sets")
            .arg(self.build_adaptation_sets())
            .arg("-seg_duration")
            .arg("4")
            .arg(self.get_output_path());

        println!("Executing FFmpeg command for video:");
        println!("{:?}", command);

        let output = command.output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("FFmpeg error: {}", error);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "FFmpeg command failed for video processing",
            ));
        }
        Ok(())
    }
}