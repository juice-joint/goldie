use std::process::Command;

#[derive(Debug)]
pub struct PitchShift {
    pub rate_multiplier: f64,
}

impl PitchShift {
    fn new(semitones: i32) -> Self {
        let rate_multiplier = 2f64.powf(semitones as f64 / 12.0);
        PitchShift { rate_multiplier }
    }
}

pub struct DashPitchShifter {
    input_file: String,
    output_file: String,
    shifts: Vec<PitchShift>,
    hw_accel: bool,
    audio_hw_accel: bool,
}

impl DashPitchShifter {
    pub fn new(
        input_file: &str,
        output_file: &str,
        semitone_range: std::ops::RangeInclusive<i32>,
        hw_accel: bool,
        audio_hw_accel: bool,
    ) -> Self {
        let shifts: Vec<PitchShift> = semitone_range.map(PitchShift::new).collect();
        DashPitchShifter {
            input_file: input_file.to_string(),
            output_file: output_file.to_string(),
            shifts,
            hw_accel,
            audio_hw_accel,
        }
    }

    fn build_filter_complex(&self) -> String {
        let num_streams = self.shifts.len();
        let mut filter = format!("[0:a]asplit={}", num_streams);
        for i in 0..num_streams {
            filter.push_str(&format!("[a{}]", i));
        }
        filter.push(';');
        
        for (i, shift) in self.shifts.iter().enumerate() {
            // Use MMAL-based pitch shifting if hardware acceleration is enabled
            if self.audio_hw_accel {
                filter.push_str(&format!(
                    " [a{}]aresample=async=1000,asetrate={}*SR,atempo={}[p{}];",
                    i,
                    shift.rate_multiplier,
                    1.0/shift.rate_multiplier, // Compensate tempo to match original speed
                    i
                ));
            } else {
                filter.push_str(&format!(
                    " [a{}]rubberband=pitch={}[p{}];",
                    i,
                    shift.rate_multiplier,
                    i
                ));
            }
        }
        
        filter.pop();
        filter
    }

    fn build_adaptation_sets(&self) -> String {
        let mut adaptation_sets = String::from("id=0,streams=0 ");
        for (i, _shift) in self.shifts.iter().enumerate() {
            adaptation_sets.push_str(&format!("id={},streams={} ", i + 1, i + 1));
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
            
            // Use hardware accelerated AAC encoder if available
            if self.audio_hw_accel {
                encodings.push("aac_mmal".to_string());
            } else {
                encodings.push("aac".to_string());
            }
            
            encodings.push("-b:a:".to_string() + &i.to_string());
            encodings.push("128k".to_string());

            // Add MMAL-specific options for hardware accelerated audio
            if self.audio_hw_accel {
                encodings.push("-profile:a:".to_string() + &i.to_string());
                encodings.push("aac_low".to_string());
            }
        }
        encodings
    }

    fn build_hw_accel_args(&self) -> Vec<String> {
        if !self.hw_accel {
            return vec![];
        }

        let mut args = vec![
            // Input hardware acceleration
            "-hwaccel".to_string(),
            "v4l2m2m".to_string(),
            "-hwaccel_device".to_string(),
            "/dev/video10".to_string(),
            // Video decoder
            "-c:v".to_string(),
            "h264_v4l2m2m".to_string(),
            // Video encoder
            "-vf".to_string(),
            "format=nv12".to_string(),
            "-c:v".to_string(),
            "h264_v4l2m2m".to_string(),
            "-b:v".to_string(),
            "2M".to_string(),
        ];

        // Add MMAL-specific options if audio hardware acceleration is enabled
        if self.audio_hw_accel {
            args.extend(vec![
                "-enable_mmal".to_string(),
                "1".to_string(),
                "-mmal_device".to_string(),
                "/dev/vchiq".to_string(),
            ]);
        }

        args
    }

    pub fn execute(&self) -> std::io::Result<()> {
        let mut command = Command::new("ffmpeg");
        
        // Add hardware acceleration arguments if enabled
        command.args(self.build_hw_accel_args());

        command
            .arg("-i")
            .arg(&self.input_file)
            .arg("-filter_complex")
            .arg(self.build_filter_complex())
            .args(self.build_stream_mappings())
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
