use tokio::sync::{mpsc, oneshot};

use crate::ytdlp::Ytdlp;


struct VideoDlActor {
    receiver: mpsc::Receiver<VideoDlActorMessage>,
    ytdlp: Ytdlp,
}

pub enum VideoDlActorMessage {
    DownloadVideo {
        yt_link: String,
        respond_to: oneshot::Sender<VideoDlActorResponse>
    }
}

pub enum VideoDlActorResponse {
    Success {
        song_name: String,
        video_file_path: String,
    },
    Fail
}

impl VideoDlActor {
    fn new(receiver: mpsc::Receiver<VideoDlActorMessage>, ytdlp: Ytdlp) -> Self {
        VideoDlActor {
            receiver: receiver,
            ytdlp: ytdlp
        }
    }

    fn handle_message(&mut self, msg: VideoDlActorMessage) {
        match msg {
            VideoDlActorMessage::DownloadVideo { 
                yt_link,
                respond_to 
            } => {
                print!("ytlink coming to actor {}", yt_link);

                self.ytdlp.fetcher.download_video_from_url(String::from("https://www.youtube.com/watch?v=3bfRnOOmXSc"), "video.mp4");

                let _ = respond_to.send(VideoDlActorResponse::Success { 
                    song_name: String::from("test"),
                    video_file_path: String::from("assets/video.mp4")
                });
            }
        }
    }
}

async fn run_video_dl_actor(mut actor: VideoDlActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg);
    }
}

#[derive(Clone)]
pub struct VideoDlActorHandle {
    sender: mpsc::Sender<VideoDlActorMessage>
}

impl VideoDlActorHandle {
    pub fn new(ytdlp: Ytdlp) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let videodl_actor = VideoDlActor::new(receiver, ytdlp);
        tokio::spawn(run_video_dl_actor(videodl_actor));

        Self { sender }
    }

    pub async fn download_video(&self, yt_link: String) -> VideoDlActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = VideoDlActorMessage::DownloadVideo { yt_link: yt_link, respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}