use std::{fs::File, io::{BufRead, BufReader, Write}, path::Path, sync::Arc};

use tokio::sync::oneshot;
use tracing::{error, info, trace};

use crate::lib::{
    dash_processor::{DashProcessor, ProcessingMode},
    yt_downloader::{VideoProcessError, YtDownloader},
};

pub enum VideoDlActorMessage {
    DownloadVideo {
        yt_link: String,
        name: String,
        pitch_shift: bool,
        respond_to: oneshot::Sender<Result<String, VideoProcessError>>,
    },
}

struct VideoDlActor {
    receiver: async_channel::Receiver<VideoDlActorMessage>,
    downloader: Arc<YtDownloader>,
    base_dir: String,
    consumer_id: u8,
}

impl VideoDlActor {
    fn new(
        receiver: async_channel::Receiver<VideoDlActorMessage>,
        base_dir: String,
        video_downloader: Arc<YtDownloader>,
        consumer_id: u8,
    ) -> Self {
        trace!("Initializing VideoDlActor consumer {}", consumer_id);
        VideoDlActor {
            receiver,
            base_dir,
            downloader: video_downloader,
            consumer_id,
        }
    }

    async fn handle_message(&mut self, msg: VideoDlActorMessage) {
        info!(
            "Consumer {} received video download message",
            self.consumer_id
        );

        match msg {
            VideoDlActorMessage::DownloadVideo {
                yt_link,
                name,
                pitch_shift,
                respond_to,
            } => {
                info!(
                    "Consumer {} starting to process video from {} to path {}",
                    self.consumer_id, yt_link, name
                );

                let video_path = format!("{}/{}", self.base_dir, name);

                info!("exists: {}", self.check_segments_and_chunks(&video_path));
                if Path::new(&video_path).exists() && self.check_segments_and_chunks(&video_path) {
                    info!(
                        "Consumer {} found existing processed video {} in path {}/{}",
                        self.consumer_id, yt_link, self.base_dir, name
                    );
                    let _ = respond_to.send(Ok(String::from("success")));
                } else {
                    let result = self
                        .process_video(&yt_link, &self.base_dir, &name, &pitch_shift, &4)
                        .await;
                    info!(
                        "Consumer {} finished processing video from {}: {:?}",
                        self.consumer_id,
                        yt_link,
                        if result.is_ok() { "success" } else { "failed" }
                    );
                    let _ = respond_to.send(result);
                }
            }
        }
    }

    fn check_segments_and_chunks(&self, base_path: &str) -> bool {
        let segments_path = format!("{}/segments.txt", base_path);
        
        // Check if segments.txt exists
        if !Path::new(&segments_path).exists() {
            trace!(
                "Consumer {} - segments.txt not found at {}",
                self.consumer_id,
                segments_path
            );
            return false;
        }

        // Read segments.txt
        let file = match File::open(&segments_path) {
            Ok(file) => file,
            Err(e) => {
                trace!(
                    "Consumer {} - Failed to open segments.txt: {}",
                    self.consumer_id,
                    e
                );
                return false;
            }
        };

        let reader = BufReader::new(file);
        let first_line = match reader.lines().next() {
            Some(Ok(line)) => line,
            _ => {
                trace!(
                    "Consumer {} - Failed to read first line from segments.txt",
                    self.consumer_id
                );
                return false;
            }
        };

        // Parse the number from segments.txt
        let segment_num = match first_line.trim().parse::<u32>() {
            Ok(num) => num,
            Err(e) => {
                trace!(
                    "Consumer {} - Failed to parse segment number from segments.txt: {}",
                    self.consumer_id,
                    e
                );
                return false;
            }
        };

        // Check if corresponding chunk file exists
        let chunk_path = format!("{}/chunk-stream1-{:05}.m4s", base_path, segment_num);
        let chunk_exists = Path::new(&chunk_path).exists();

        trace!(
            "Consumer {} - Checking for chunk file: {} - {}",
            self.consumer_id,
            chunk_path,
            if chunk_exists { "found" } else { "not found" }
        );

        chunk_exists
    }

    async fn process_video(
        &self,
        yt_link: &str,
        base_dir: &str,
        name: &str,
        pitch_shift: &bool,
        segment_duration: &u32,
    ) -> Result<String, VideoProcessError> {
        trace!(
            "Consumer {} starting download of {}",
            self.consumer_id,
            yt_link
        );
        let video_metadata = self.downloader.download(yt_link, base_dir, name).await?;
        let (dir, file_name, extension, duration_seconds) = (
            video_metadata.directory,
            video_metadata.filename,
            video_metadata.extension,
            video_metadata.duration_seconds,
        );

        let duration_file_path = format!("{}/segments.txt", dir);
        match File::create(&duration_file_path) {
            Ok(mut file) => match write!(
                file,
                "{}",
                (duration_seconds / (*segment_duration as f64)).ceil()
            ) {
                Ok(_) => {
                    trace!(
                        "Consumer {} wrote duration {} seconds to {}",
                        self.consumer_id,
                        duration_seconds,
                        duration_file_path
                    );
                }
                Err(e) => {
                    trace!(
                        "Consumer {} failed to write duration to {}: {}",
                        self.consumer_id,
                        duration_file_path,
                        e
                    );
                    return Err(VideoProcessError::PitchShiftError(format!(
                        "Failed to write duration: {}",
                        e
                    )));
                }
            },
            Err(e) => {
                trace!(
                    "Consumer {} failed to create duration file {}: {}",
                    self.consumer_id,
                    duration_file_path,
                    e
                );
                return Err(VideoProcessError::PitchShiftError(format!(
                    "Failed to create duration file: {}",
                    e
                )));
            }
        }

        trace!(
            "Consumer {} completed download. Dir: {}, File: {}.{}",
            self.consumer_id,
            dir,
            file_name,
            extension
        );

        let dash_processor = DashProcessor::new(4);
        let mode;

        if *pitch_shift {
            trace!(
                "Consumer {} starting dash processing with pitch shifting for {}",
                self.consumer_id,
                file_name
            );
            mode = ProcessingMode::PitchShift(vec![--3, -2, -1, 0, 1, 2, 3])
        } else {
            trace!(
                "Consumer {} starting dash processing with no pitch shifting for {}",
                self.consumer_id,
                file_name
            );
            mode = ProcessingMode::Copy;
        }

        match dash_processor.execute(
            &format!("{}/{}.{}", dir, file_name, extension),
            &format!("{}/{}.mpd", dir, file_name),
            &mode,
        ) {
            Ok(_) => {
                trace!(
                    "Consumer {} completed pitch shifting for {}",
                    self.consumer_id,
                    file_name
                );
                Ok(format!("{}/{}.{}", dir, file_name, extension))
            }
            Err(e) => {
                trace!(
                    "Consumer {} failed pitch shifting for {}: {}",
                    self.consumer_id,
                    file_name,
                    e
                );
                Err(VideoProcessError::PitchShiftError(format!(
                    "Pitch shift failed: {}",
                    e
                )))
            }
        }
    }
}

async fn run_video_dl_actor(mut actor: VideoDlActor) {
    info!(
        "Starting video download actor consumer {}",
        actor.consumer_id
    );
    loop {
        trace!(
            "Consumer {} waiting for message. Channel capacity: {}, len: {}",
            actor.consumer_id,
            actor.receiver.capacity().unwrap(),
            actor.receiver.len()
        );

        match actor.receiver.recv().await {
            Ok(msg) => {
                trace!("Total receiver count: {}", actor.receiver.receiver_count());

                trace!(
                    "Consumer {} received message. Channel capacity: {}, len: {}",
                    actor.consumer_id,
                    actor.receiver.capacity().unwrap(),
                    actor.receiver.len()
                );
                actor.handle_message(msg).await;
                trace!(
                    "Consumer {} completed processing. Channel capacity: {}, len: {}",
                    actor.consumer_id,
                    actor.receiver.capacity().unwrap(),
                    actor.receiver.len()
                );
            }
            Err(e) => {
                error!(
                    "Consumer {} channel closed, shutting down: {}",
                    actor.consumer_id, e
                );
                break;
            }
        }
    }
    info!("Consumer {} shutting down", actor.consumer_id);
}

#[derive(Clone)]
pub struct VideoDlActorHandle {
    sender: async_channel::Sender<VideoDlActorMessage>,
}

impl VideoDlActorHandle {
    pub fn new(base_dir: String, yt_downloader: Arc<YtDownloader>) -> Self {
        trace!("Initializing VideoDlActorHandle");
        let (sender, receiver) = async_channel::bounded(100);
        trace!(
            "Created channel with capacity: {}",
            sender.capacity().unwrap()
        );

        const NUM_CONSUMERS: u8 = 5;
        trace!("Starting {} consumers", NUM_CONSUMERS);
        for consumer_id in 0..NUM_CONSUMERS {
            trace!("Spawning consumer {}", consumer_id);
            let actor = VideoDlActor::new(
                receiver.clone(),
                base_dir.clone(),
                yt_downloader.clone(),
                consumer_id,
            );
            tokio::spawn(run_video_dl_actor(actor));
        }
        trace!("All consumers spawned");
        trace!("Total receiver count: {}", receiver.receiver_count());

        Self { sender }
    }

    pub async fn download_video(
        &self,
        yt_link: String,
        name: String,
        pitch_shift: bool,
    ) -> Result<String, VideoProcessError> {
        trace!(
            "Requesting video download for {} (channel len: {})",
            yt_link,
            self.sender.len()
        );

        let (send, recv) = oneshot::channel();
        let msg = VideoDlActorMessage::DownloadVideo {
            yt_link: yt_link.clone(),
            name: name.clone(),
            pitch_shift: pitch_shift.clone(),
            respond_to: send,
        };

        trace!(
            "Sending download request for {} to video download actor (channel len: {})",
            yt_link,
            self.sender.len()
        );
        let _ = self.sender.send(msg).await;

        trace!(
            "Message sent for {}. Channel status - len: {}, capacity: {}",
            yt_link,
            self.sender.len(),
            self.sender.capacity().unwrap()
        );

        trace!("Awaiting response for {}", yt_link);
        let result = recv.await.expect("Actor task has been killed");
        trace!(
            "Received response for {}: {:?}",
            yt_link,
            if result.is_ok() { "success" } else { "failed" }
        );
        result
    }
}
