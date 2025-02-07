use std::process::Command;

#[derive(Debug)]
pub struct PitchShift {
    pub semitones: i32,
    pub rate_multiplier: f64,
    pub tempo_multiplier: f64,
}

impl PitchShift {
    fn new(semitones: i32) -> Self {
        // Calculate rate multiplier using 2^(n/12) formula
        let rate_multiplier = 2f64.powf(semitones as f64 / 12.0);
        // Tempo multiplier is the inverse to maintain duration
        let tempo_multiplier = 1.0 / rate_multiplier;

        PitchShift {
            semitones,
            rate_multiplier,
            tempo_multiplier,
        }
    }
}

pub struct DashPitchShifter {
    input_file: String,
    output_file: String,
    shifts: Vec<PitchShift>,
}

impl DashPitchShifter {
    pub fn new(
        input_file: &str,
        output_file: &str,
        semitone_range: std::ops::RangeInclusive<i32>,
    ) -> Self {
        let shifts: Vec<PitchShift> = semitone_range.map(PitchShift::new).collect();

        DashPitchShifter {
            input_file: input_file.to_string(),
            output_file: output_file.to_string(),
            shifts,
        }
    }

    fn build_filter_complex(&self) -> String {
        let num_streams = self.shifts.len();
        // Create asplit filter
        let mut filter = format!("[0:a]asplit={}", num_streams);
        for i in 0..num_streams {
            filter.push_str(&format!("[a{}]", i));
        }
        filter.push(';');
       
        // Add rubberband pitch shift filters for each stream
        // Convert semitones to pitch multiplier using the rate_multiplier
        for (i, shift) in self.shifts.iter().enumerate() {
            filter.push_str(&format!(
                " [a{}]rubberband=pitch={}[p{}];",
                i, shift.rate_multiplier, i
            ));
        }
       
        // Remove the last semicolon
        filter.pop();
        filter
    }

    fn build_adaptation_sets(&self) -> String {
        let mut adaptation_sets = String::from("id=0,streams=0 ");

        for (i, shift) in self.shifts.iter().enumerate() {
            adaptation_sets.push_str(&format!("id={},streams={} ", i + 1, i + 1,));
        }

        adaptation_sets.trim().to_string()
    }

    fn build_stream_mappings(&self) -> Vec<String> {
        let mut mappings = vec!["-map".to_string(), "0:v".to_string()];

        for i in 0..self.shifts.len() {
            mappings.push("-map".to_string());
            mappings.push(format!("[p{}]", i));
        }

        mappings
    }

    fn build_audio_encodings(&self) -> Vec<String> {
        let mut encodings = Vec::new();

        for i in 0..self.shifts.len() {
            encodings.push("-c:a:".to_string() + &i.to_string());
            encodings.push("aac".to_string());
            encodings.push("-b:a:".to_string() + &i.to_string());
            encodings.push("128k".to_string());
        }

        encodings
    }

    pub fn execute(&self) -> std::io::Result<()> {
        let mut command = Command::new("ffmpeg");

        command
            .arg("-i")
            .arg(&self.input_file)
            .arg("-filter_complex")
            .arg(self.build_filter_complex())
            .args(self.build_stream_mappings())
            .arg("-c:v")
            .arg("copy")
            .args(self.build_audio_encodings())
            .arg("-f")
            .arg("dash")
            .arg("-adaptation_sets")
            .arg(self.build_adaptation_sets())
            .arg("-seg_duration")
            .arg("4")
            .arg(&self.output_file);

        println!("Executing FFmpeg command:");
        println!("{:?}", command);

        let output = command.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("FFmpeg error: {}", error);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "FFmpeg command failed",
            ));
        }

        Ok(())
    }
}