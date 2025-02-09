use std::process::Command;

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
            semitones 
        }
    }
}

pub struct DashPitchShifter {
    input_file: String,
    output_dir: String,
    shifts: Vec<PitchShift>,
}

impl DashPitchShifter {
    pub fn new(
        input_file: &str,
        output_dir: &str,
        semitone_range: std::ops::RangeInclusive<i32>,
    ) -> Self {
        let shifts: Vec<PitchShift> = semitone_range.map(PitchShift::new).collect();

        DashPitchShifter {
            input_file: input_file.to_string(),
            output_dir: output_dir.to_string(),
            shifts,
        }
    }

    fn build_command_for_shift(&self, shift: &PitchShift) -> Command {
        let mut command = Command::new("ffmpeg");
        let output_file = format!("{}/pitch{}.mpd", self.output_dir, shift.semitones);

        // Base command setup
        command
            .arg("-i")
            .arg(&self.input_file)
            .arg("-threads")
            .arg("16")
            .arg("-thread_type")
            .arg("frame");

        if shift.semitones == 0 {
            // Original pitch - no filter needed
            command
                .arg("-map")
                .arg("0:v")
                .arg("-map")
                .arg("0:a")
                .arg("-c:v")
                .arg("copy")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("128k");
        } else {
            // Pitch-shifted version
            command
                .arg("-filter_threads")
                .arg("16")
                .arg("-filter_complex_threads")
                .arg("16")
                .arg("-filter_complex")
                .arg(format!("[0:a]rubberband=pitch={}:threads=16[p0]", shift.rate_multiplier))
                .arg("-map")
                .arg("0:v")
                .arg("-map")
                .arg("[p0]")
                .arg("-c:v")
                .arg("copy")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("128k");
        }

        // Common output settings
        command
            .arg("-f")
            .arg("dash")
            .arg("-adaptation_sets")
            .arg("id=0,streams=0 id=1,streams=1")
            .arg("-seg_duration")
            .arg("4")
            .arg(output_file);

        command
    }

    pub fn execute(&self) -> std::io::Result<()> {
        for shift in &self.shifts {
            let mut command = self.build_command_for_shift(shift);
            
            println!("Executing FFmpeg command for {} semitones:", shift.semitones);
            println!("{:?}", command);

            let output = command.output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("FFmpeg error for {} semitones: {}", shift.semitones, error);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("FFmpeg command failed for {} semitones", shift.semitones),
                ));
            }
        }

        Ok(())
    }
}