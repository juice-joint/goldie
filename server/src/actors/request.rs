use tokio::sync::{mpsc, oneshot};

use crate::{actors::videodl::VideoDlActorHandle, queue::{PlayableSong, SongActorHandle, SongActorResponse}};

use super::videodl::VideoDlActorResponse;

struct RequestActor {
    receiver: mpsc::Receiver<RequestActorMessage>,
    videodl_actor_handle: VideoDlActorHandle,
    song_actor_handle: SongActorHandle
}

pub enum RequestActorMessage {
    QueueSong {
        respond_to: oneshot::Sender<RequestActorResponse>
    },
    PlayNext {
        respond_to: oneshot::Sender<RequestActorResponse>
    },
    Remove {
        respond_to: oneshot::Sender<RequestActorResponse>
    },
    ReOrder {
        respond_to: oneshot::Sender<RequestActorResponse>
    },
    SongList {
        respond_to: oneshot::Sender<RequestActorResponse>
    }
}

#[derive(Debug)]
pub enum RequestActorResponse {
    QueueSuccess,
    PlayNextSuccess {
        video_file_path: String
    },
    Fail
}

impl RequestActor {
    fn new(
        receiver: mpsc::Receiver<RequestActorMessage>, 
        videodl_actor_handle: VideoDlActorHandle,
        song_actor_handle: SongActorHandle
    ) -> Self {
        RequestActor {
            receiver: receiver,
            videodl_actor_handle: videodl_actor_handle,
            song_actor_handle: song_actor_handle
        }
    }

    async fn handle_message(&mut self, msg: RequestActorMessage) {
        match msg {
            RequestActorMessage::QueueSong { respond_to } => {
                let videodl_response: super::videodl::VideoDlActorResponse = self.videodl_actor_handle.download_video(String::from("test")).await;
                match videodl_response {
                    VideoDlActorResponse::Success { song_name, video_file_path } => {
                        self.song_actor_handle.queue_song(PlayableSong::new(song_name, video_file_path)).await;
                    },
                    VideoDlActorResponse::Fail => {

                    }
                }

                let _ = respond_to.send(RequestActorResponse::QueueSuccess);
            }
            RequestActorMessage::PlayNext { respond_to } => {
                let song_actor_response = self.song_actor_handle.pop_song().await;
                match song_actor_response {
                    SongActorResponse::CurrentSong { optional_song } => {
                        match optional_song {
                            Some(song) => {
                                let _ = respond_to.send(RequestActorResponse::PlayNextSuccess { video_file_path: song.video_file_path });
                            }
                            None => {
                                let _ = respond_to.send(RequestActorResponse::Fail);
                            }
                        }
                    }
                    SongActorResponse::Success => todo!(),
                    SongActorResponse::Fail => todo!(),
                }
            },
            RequestActorMessage::Remove { respond_to } => todo!(),
            RequestActorMessage::ReOrder { respond_to } => todo!(),
            RequestActorMessage::SongList { respond_to } => {
                let song_actor_response = self.song_actor_handle.pop_song().await;
                match song_actor_response {
                    SongActorResponse::CurrentSong { optional_song } => {
                        match optional_song {
                            Some(song) => {
                                let _ = respond_to.send(RequestActorResponse::PlayNextSuccess { video_file_path: song.video_file_path });
                            }
                            None => {
                                let _ = respond_to.send(RequestActorResponse::Fail);
                            }
                        }
                    }
                    SongActorResponse::Success => todo!(),
                    SongActorResponse::Fail => todo!(),
                }
            },
        }
    }
}

async fn run_request_actor(mut actor: RequestActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}

#[derive(Clone)]
pub struct RequestActorHandle {
    sender: mpsc::Sender<RequestActorMessage>
}

impl RequestActorHandle {
    pub fn new(videodl_actor_handle: VideoDlActorHandle, song_actor_handle: SongActorHandle) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let request_actor = RequestActor::new(receiver, videodl_actor_handle, song_actor_handle);
        tokio::spawn(run_request_actor(request_actor));

        Self { sender }
    }

    pub async fn queue_song(&self) -> RequestActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = RequestActorMessage::QueueSong { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn play_next_song(&self) -> RequestActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = RequestActorMessage::PlayNext { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed") 
    }

    pub async fn song_list(&self) -> RequestActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = RequestActorMessage::SongList { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed") 
    }
}