use tokio::process::Command;
use futures_util::future::join_all;
use std::path::Path;
use tokio::fs;
use tokio::sync::Semaphore;
use std::sync::Arc;

#[derive(Debug)]
pub struct PitchShift {
    pub rate_multiplier: f64,
    pub semitones: i32,
}

impl PitchShift {
    fn new(semitones: i32) -> Self {
        // Calculate rate multiplier using 2^(n/12) formula
        let rate_multiplier = 2f64.powf(semitones as f64 / 12.0);
        PitchShift { 
            rate_multiplier,
            semitones,
        }
    }
}

pub struct DashPitchShifter {
    input_file: String,
    output_dir: String,
    shifts: Vec<PitchShift>,
    max_concurrent_tasks: usize,
}

impl DashPitchShifter {
    pub fn new(
        input_file: &str,
        output_dir: &str,
        semitone_range: std::ops::RangeInclusive<i32>,
        max_concurrent_tasks: usize,
    ) -> Self {
        let shifts: Vec<PitchShift> = semitone_range.map(PitchShift::new).collect();

        DashPitchShifter {
            input_file: input_file.to_string(),
            output_dir: output_dir.to_string(),
            shifts,
            max_concurrent_tasks,
        }
    }

    fn build_filter_complex(&self, shift: &PitchShift) -> String {
        format!("[0:a]rubberband=pitch={}:threads=16[p0]", shift.rate_multiplier)
    }

    fn build_adaptation_sets(&self) -> String {
        "id=0,streams=0".to_string()
    }

    fn build_stream_mappings(&self) -> Vec<String> {
        vec!["-map".to_string(), "[p0]".to_string()]
    }

    fn build_audio_encodings(&self) -> Vec<String> {
        vec![
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "128k".to_string(),
        ]
    }

    fn get_output_path(&self, semitones: i32) -> String {
        let index = match semitones {
            -3 => 7,
            -2 => 6,
            -1 => 5,
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 4,
            _ => 1,
        };

        format!("{}/pitch{}/stream.mpd", self.output_dir, index)
    }

    async fn ensure_output_directories(&self) -> std::io::Result<()> {
        // Create main output directory if it doesn't exist
        if !Path::new(&self.output_dir).exists() {
            fs::create_dir_all(&self.output_dir).await?;
        }

        // Create pitch-specific subdirectories
        let mut id = 1;
        for _shift in &self.shifts {
            let pitch_dir = format!("{}/pitch{}", self.output_dir, id);
            if !Path::new(&pitch_dir).exists() {
                fs::create_dir_all(&pitch_dir).await?;
            }
            id += 1;
        }

        Ok(())
    }

    async fn process_shift(&self, shift: &PitchShift) -> std::io::Result<()> {
        let mut command = Command::new("ffmpeg");
        command
            .arg("-i")
            .arg(&self.input_file)
            .arg("-threads")
            .arg("16")
            .arg("-filter_threads")
            .arg("16")
            .arg("-filter_complex_threads")
            .arg("16")
            .arg("-thread_type")
            .arg("frame")
            .arg("-filter_complex")
            .arg(self.build_filter_complex(shift))
            .args(self.build_stream_mappings())
            .args(self.build_audio_encodings())
            .arg("-f")
            .arg("dash")
            .arg("-adaptation_sets")
            .arg(self.build_adaptation_sets())
            .arg("-seg_duration")
            .arg("4")
            .arg(self.get_output_path(shift.semitones));

        println!("Executing FFmpeg command for {} semitones:", shift.semitones);
        println!("{:?}", command);

        let output = command.output().await?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("FFmpeg error: {}", error);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("FFmpeg command failed for {} semitones", shift.semitones),
            ));
        }
        Ok(())
    }

    async fn process_shift_with_semaphore(
        &self,
        shift: &PitchShift,
        semaphore: Arc<Semaphore>,
    ) -> std::io::Result<()> {
        // Acquire semaphore permit - will wait here if we're at max concurrent tasks
        let _permit = semaphore.acquire().await.unwrap();
        
        println!("Starting processing for {} semitones", shift.semitones);
        let result = self.process_shift(shift).await;
        println!("Finished processing for {} semitones", shift.semitones);
        
        // Permit is automatically released when _permit is dropped at end of scope
        result
    }

    pub async fn execute(&self) -> std::io::Result<()> {
        // Ensure output directories exist before processing
        self.ensure_output_directories().await?;

        // Create semaphore to limit concurrent tasks
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_tasks));

        // Create futures for all shifts, but with semaphore control
        let futures = self.shifts.iter().map(|shift| {
            self.process_shift_with_semaphore(shift, Arc::clone(&semaphore))
        });

        // Execute all futures with controlled concurrency
        let results = join_all(futures).await;
        
        // Check for any errors
        let errors: Vec<_> = results
            .into_iter()
            .enumerate()
            .filter_map(|(i, r)| r.err().map(|e| (self.shifts[i].semitones, e)))
            .collect();

        if !errors.is_empty() {
            let error_msg = errors
                .iter()
                .map(|(semitones, e)| format!("Semitones {}: {}", semitones, e))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Multiple FFmpeg commands failed:\n{}", error_msg),
            ));
        }

        Ok(())
    }
}