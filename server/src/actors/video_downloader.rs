use std::{path::Path, sync::Arc};

use tokio::sync::oneshot;

use crate::lib::{
    pitch_shifter::DashPitchShifter, 
    yt_downloader::{VideoProcessError, YtDownloader}
};

pub enum VideoDlActorMessage {
    DownloadVideo {
        yt_link: String,
        name: String,
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
        println!("Initializing VideoDlActor consumer {}", consumer_id);
        VideoDlActor {
            receiver,
            base_dir,
            downloader: video_downloader,
            consumer_id,
        }
    }

    async fn handle_message(&mut self, msg: VideoDlActorMessage) {
        println!("Consumer {} received video download message", self.consumer_id);

        match msg {
            VideoDlActorMessage::DownloadVideo {
                yt_link,
                name,
                respond_to,
            } => {
                println!("Consumer {} starting to process video from {} to path {}", 
                    self.consumer_id, yt_link, name);

                if Path::new(&format!("{}/{}", self.base_dir, name)).exists() {
                    println!("Consumer {} found existing processed video {} in path {}/{}", 
                        self.consumer_id, yt_link, self.base_dir, name);
                    let _ = respond_to.send(Ok(String::from("success")));
                } else {
                    let result = self.process_video(&yt_link, &self.base_dir, &name).await;
                    println!("Consumer {} finished processing video from {}: {:?}", 
                        self.consumer_id, yt_link, 
                        if result.is_ok() { "success" } else { "failed" });
                    let _ = respond_to.send(result);
                }
            }
        }
    }

    async fn process_video(&self, yt_link: &str, base_dir: &str, name: &str) -> Result<String, VideoProcessError> {
        println!("Consumer {} starting download of {}", self.consumer_id, yt_link);
        let (dir, file_name, extension) = self.downloader.download(yt_link, base_dir, name).await?;
        println!("Consumer {} completed download. Dir: {}, File: {}.{}", 
            self.consumer_id, dir, file_name, extension);

        println!("Consumer {} starting pitch shifting for {}", self.consumer_id, file_name);
        let pitch_shifter = DashPitchShifter::new(
            &format!("{}/{}.{}", dir, file_name, extension),
            &format!("{}/{}.mpd", dir, file_name),
            -3..=3
        );

        match pitch_shifter.execute() {
            Ok(_) => {
                println!("Consumer {} completed pitch shifting for {}", self.consumer_id, file_name);
                Ok(format!("{}/{}.{}", dir, file_name, extension))
            },
            Err(e) => {
                println!("Consumer {} failed pitch shifting for {}: {}", 
                    self.consumer_id, file_name, e);
                Err(VideoProcessError::PitchShiftError(format!("Pitch shift failed: {}", e)))
            }
        }
    }
}

async fn run_video_dl_actor(mut actor: VideoDlActor) {
    println!("Starting video download actor consumer {}", actor.consumer_id);
    loop {
        println!("Consumer {} waiting for message. Channel capacity: {}, len: {}", 
            actor.consumer_id, 
            actor.receiver.capacity().unwrap(),
            actor.receiver.len());
            
        match actor.receiver.recv().await {
            Ok(msg) => {

                println!("Total count: {}", actor.receiver.receiver_count());

                println!("Consumer {} received message. Channel capacity: {}, len: {}", 
                    actor.consumer_id, 
                    actor.receiver.capacity().unwrap(),
                    actor.receiver.len());
                actor.handle_message(msg).await;
                println!("Consumer {} completed processing. Channel capacity: {}, len: {}", 
                    actor.consumer_id,
                    actor.receiver.capacity().unwrap(),
                    actor.receiver.len());
                    
            },
            Err(e) => {
                println!("Consumer {} channel closed, shutting down: {}", actor.consumer_id, e);
                break;
            }
        }
    }
    println!("Consumer {} shutting down", actor.consumer_id);
}

#[derive(Clone)]
pub struct VideoDlActorHandle {
    sender: async_channel::Sender<VideoDlActorMessage>,
}

impl VideoDlActorHandle {
    pub fn new(base_dir: String, yt_downloader: Arc<YtDownloader>) -> Self {
        println!("Initializing VideoDlActorHandle");
        let (sender, receiver) = async_channel::bounded(100);
        println!("Created channel with capacity: {}", sender.capacity().unwrap());

        const NUM_CONSUMERS: u8 = 5;
        println!("Starting {} consumers", NUM_CONSUMERS);
        for consumer_id in 0..NUM_CONSUMERS {
            println!("Spawning consumer {}", consumer_id);
            let actor = VideoDlActor::new(receiver.clone(), base_dir.clone(), yt_downloader.clone(), consumer_id);
            tokio::spawn(run_video_dl_actor(actor));
        }
        println!("All consumers spawned");
        println!("Total count: {}", receiver.receiver_count());

        Self { sender }
    }

    pub async fn download_video(&self, yt_link: String, file_path: String) -> Result<String, VideoProcessError> {
        println!("Requesting video download for {} (channel len: {})", 
            yt_link, 
            self.sender.len());
            
        let (send, recv) = oneshot::channel();
        let msg = VideoDlActorMessage::DownloadVideo {
            yt_link: yt_link.clone(),
            name: file_path.clone(),
            respond_to: send,
        };

        println!("Sending download request for {} to video download actor (channel len: {})", 
            yt_link,
            self.sender.len());
        let _ = self.sender.send(msg).await;
        
        println!("Message sent for {}. Channel status - len: {}, capacity: {}", 
            yt_link,
            self.sender.len(),
            self.sender.capacity().unwrap());
            
        println!("Awaiting response for {}", yt_link);
        let result = recv.await.expect("Actor task has been killed");
        println!("Received response for {}: {:?}", 
            yt_link, 
            if result.is_ok() { "success" } else { "failed" });
        result
    }
}