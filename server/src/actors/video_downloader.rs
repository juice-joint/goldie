use std::process::Command;

use tokio::sync::{mpsc, oneshot};

use crate::ytdlp::Ytdlp;


struct VideoDlActor {
    receiver: mpsc::Receiver<VideoDlActorMessage>,
    ytdlp: Ytdlp,
}

pub enum VideoDlActorMessage {
    DownloadVideo {
        yt_link: String,
        respond_to: oneshot::Sender<DownloadVideoResponse>
    }
}

pub enum DownloadVideoResponse {
    Success {
        song_name: String,
        video_file_path: String
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
                let args = [
                    "-f",
                    "bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
                    "-o",
                    &format!("{}/%(title)s.%(ext)s", "./assets"),
                    "--restrict-filenames",
                    "--get-filename",
                    "--no-simulate",
                    &yt_link
                ];
                let output = Command::new(r"C:\Users\jared\.local\bin\yt-dlp.exe")
                    .args(&args)
                    .output();
                match output {
                    Ok(output) => {
                        if output.status.success() {
                            if let Ok(filename) = String::from_utf8(output.stdout) {
                                let filename = filename.trim();
                                if let Some((name, ext)) = filename.rsplit_once(".") {
                                    let todo_remove = format!("{}.f135", name);
                                    println!("Download successful! Saved as: {}", todo_remove);
                                    let _ = respond_to.send(DownloadVideoResponse::Success { 
                                        song_name: String::from("test"),
                                        video_file_path: format!("{}", todo_remove.to_string())
                                    });
                                } else {
                                    println!("Failed to parse filename into name and extension.");
                                }
                            } else {
                                eprintln!("Error: Unable to parse filename from yt-dlp output");
                            }
                        } else {
                            eprintln!("Error downloading video using yt-dlp")
                        }
                    }
                    Err(error) => {
                        eprintln!("Failed to execute yt-dlp: {}", error);
                    }
                }
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

    pub async fn download_video(&self, yt_link: String) -> DownloadVideoResponse {
        let (send, recv) = oneshot::channel();
        let msg = VideoDlActorMessage::DownloadVideo { yt_link: yt_link, respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}