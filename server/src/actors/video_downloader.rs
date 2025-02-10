use std::{fs, path::Path, sync::Arc};

use quick_xml::{de::from_str, se::to_string};
use thiserror::Error;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::lib::{
    pitch_shifter::DashPitchShifter, video_extractor::DashVideoProcessor, xml_mpd::{self, XmlMpdUtil, MPD}, yt_downloader::{VideoProcessError, YtDownloader}
};

pub enum VideoDlActorMessage {
    DownloadVideo {
        yt_link: String,
        file_path: String,
        respond_to: oneshot::Sender<Result<String, VideoProcessError>>,
    },
}

pub enum DownloadVideoResponse {
    Success { video_file_path: String },
    Fail,
}

struct VideoDlActor {
    receiver: async_channel::Receiver<VideoDlActorMessage>,
    downloader: Arc<YtDownloader>,
}

impl VideoDlActor {
    fn new(
        receiver: async_channel::Receiver<VideoDlActorMessage>,
        video_downloader: Arc<YtDownloader>,
    ) -> Self {
        VideoDlActor {
            receiver,
            downloader: video_downloader,
        }
    }

    async fn handle_message(&mut self, msg: VideoDlActorMessage) {
        println!("received video download message");

        match msg {
            VideoDlActorMessage::DownloadVideo {
                yt_link,
                file_path,
                respond_to,
            } => {
                let result = self.process_video(&yt_link, &file_path).await;
                let _ = respond_to.send(result);
            }
        }
    }

    async fn process_video(&self, yt_link: &str, file_path: &str) -> Result<String, VideoProcessError> {
        // Download the video
        let (dir, file_name, extension) = self.downloader.download(yt_link, &file_path).await?;
        println!("Download successful! Saved in: {}", dir);

        println!("{}", dir);
        println!("{}", file_name);
        println!("{}", extension);

        // Process with pitch shifting
        let pitch_shifter = DashPitchShifter::new(
            &format!("{}/{}.{}", dir, file_name, extension),
            &format!("{}", dir),
            -3..=3,
            2
        );

        pitch_shifter.execute().await.map_err(|e| {
            VideoProcessError::PitchShiftError(format!("Pitch shift failed: {}", e))
        })?;

        let video_extractor = DashVideoProcessor::new(
            &format!("{}/{}.{}", dir, file_name, extension), 
            &format!("{}", dir));

        video_extractor.execute().map_err(|e| {
            VideoProcessError::VideoExtractError(format!("Video extraction failed: {}", e))
        })?;

        println!("starting renaming");

        rename_stream_files(Path::new(&dir))?;

        println!("done renaming");

        let mpd_files = XmlMpdUtil::find_mpd_files(Path::new(&dir));

        let mut adaptation_sets = Vec::new();

        let mut adapt_id = 1;
        for (index, file_path) in mpd_files?.iter().enumerate() {
            println!("{:?}", file_path);

            let xml = fs::read_to_string(file_path)?;
            
            let mpd: MPD = from_str(&xml).expect("hi");

            adaptation_sets.extend(mpd.period.adaptation_sets.into_iter().map(|mut adaptation_set| {

                if adaptation_set.content_type == "video" {
                    adaptation_set.id = 0.to_string();
                    adaptation_set.representation.id = 0.to_string();

                    let init_file_path = adaptation_set.representation.segment_template.initialization;
                    let media_files_path = adaptation_set.representation.segment_template.media;

                    adaptation_set.representation.segment_template.initialization = format!("video/{}", init_file_path);
                    adaptation_set.representation.segment_template.media = format!("video/{}", media_files_path);
                } else {
                    adaptation_set.id = adapt_id.to_string();
                    adaptation_set.representation.id = adapt_id.to_string();

                    let init_file_path = adaptation_set.representation.segment_template.initialization;
                    let media_files_path = adaptation_set.representation.segment_template.media;
                    
                    adaptation_set.representation.segment_template.initialization = format!("pitch{}/{}", adapt_id, init_file_path);
                    adaptation_set.representation.segment_template.media = format!("pitch{}/{}", adapt_id, media_files_path);
                }

                adaptation_set
            }));

            adapt_id += 1;
        }

        for adaptation_set in &adaptation_sets {
            println!("{:?}", adaptation_set);
            println!("\n------------------");
        }

        let start_xml = r#"<?xml version="1.0" encoding="utf-8"?>
        <MPD xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
            xmlns="urn:mpeg:dash:schema:mpd:2011"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            xsi:schemaLocation="urn:mpeg:DASH:schema:MPD:2011 http://standards.iso.org/ittf/PubliclyAvailableStandards/MPEG-DASH_schema_files/DASH-MPD.xsd"
            profiles="urn:mpeg:dash:profile:isoff-live:2011"
            type="static"
            mediaPresentationDuration="PT4M17.8S"
            maxSegmentDuration="PT4.0S"
            minBufferTime="PT14.0S">
            <ProgramInformation>
            </ProgramInformation>
            <ServiceDescription id="0">
            </ServiceDescription>
            <Period id="0" start="PT0.0S">"#;
        
        // Create the complete XML
        let mut output = String::new();
        output.push_str(start_xml);
    
        // Add each AdaptationSet
        for adaptation_set in adaptation_sets.iter() {
            let adaptation_set_xml = to_string(&adaptation_set).unwrap();
            output.push_str("\n        "); // Indentation
            output.push_str(&adaptation_set_xml);
        }
    
        // Add closing tags
        output.push_str("\n    </Period>\n</MPD>");
    
        // Write to file
        fs::write(format!("{}/{}.mpd", dir, file_name), output)?;
        println!("Written to output.mpd");    

        println!("done!");

        Ok(format!("{}/{}.{}", dir, file_name, extension))
    }
}

fn rename_stream_files(dir: &Path) -> Result<(), std::io::Error> {
    let output_dir = dir;

    if !output_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Output directory not found: {}", output_dir.display()),
        ));
    }

    // Read all pitch directories
    for entry in fs::read_dir(&output_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            if dir_name.starts_with("pitch") {
                // Extract the pitch number
                if let Some(pitch_num) = dir_name.strip_prefix("pitch").and_then(|n| n.parse::<i32>().ok()) {
                    println!("Processing directory: {}", dir_name);
                    
                    // Read all files in the pitch directory
                    for file in fs::read_dir(&path)? {
                        let file = file?;
                        let file_path = file.path();
                        
                        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                            if file_name.contains("stream0") {
                                // Create new filename by replacing stream0 with streamN
                                let new_name = file_name.replace("stream0", &format!("stream{}", pitch_num));
                                let new_path = file_path.with_file_name(new_name);
                                
                                println!("Renaming {} to {}", file_name, new_path.file_name().unwrap().to_str().unwrap());
                                fs::rename(&file_path, &new_path)?;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn run_video_dl_actor(mut actor: VideoDlActor) {
    while let Ok(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}

#[derive(Clone)]
pub struct VideoDlActorHandle {
    sender: async_channel::Sender<VideoDlActorMessage>,
}

impl VideoDlActorHandle {
    pub fn new(yt_downloader: Arc<YtDownloader>) -> Self {
        let (sender, receiver) = async_channel::bounded(5);

        // TODO grab from settings descriptor
        const NUM_CONSUMERS: u8 = 1;
        for _ in 0..NUM_CONSUMERS {
            let actor = VideoDlActor::new(receiver.clone(), yt_downloader.clone());
            tokio::spawn(run_video_dl_actor(actor));
        }

        Self { sender }
    }

    pub async fn download_video(&self, yt_link: String, file_path: String) -> Result<String, VideoProcessError> {
        let (send, recv) = oneshot::channel();
        let msg = VideoDlActorMessage::DownloadVideo {
            yt_link,
            file_path,
            respond_to: send,
        };

        println!("sending download video message to videodl actor");

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}
