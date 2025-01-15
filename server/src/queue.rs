use std::{collections::VecDeque, sync::Arc, usize};

use tokio::sync::{self, mpsc, oneshot};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct PlayableSong {
    pub name: String,
    pub video_file_path: String,
    pub uuid: Uuid,
}

impl PlayableSong {
    pub fn new(name: String, video_file_path: String) -> Self {
        PlayableSong {
            name: name.to_string(),
            video_file_path: video_file_path.to_string(),
            uuid: Uuid::new_v4(),
        }
    }
}

impl ToString for PlayableSong {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl PartialEq for PlayableSong {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

struct SongActor {
    receiver: mpsc::Receiver<SongActorMessage>,
    song_deque: VecDeque<PlayableSong>,
    current_song: Option<PlayableSong>,
    sse_broadcaster: Arc<sync::broadcast::Sender<PlayableSong>>
}

pub enum SongActorMessage {
    QueueSong {
        playable_song: PlayableSong,
        respond_to: oneshot::Sender<SongActorResponse>,
    },
    RemoveSong {
        song_uuid: Uuid,
        respond_to: oneshot::Sender<SongActorResponse>,
    },
    PopSong {
        respond_to: oneshot::Sender<SongActorResponse>,
    },
    Reposition {
        song_uuid: Uuid,
        position: usize,
        respond_to: oneshot::Sender<SongActorResponse>,
    },
    Current {
        respond_to: oneshot::Sender<SongActorResponse>,
    },
    
}

pub enum SongActorResponse {
    CurrentSong {
        optional_song: Option<PlayableSong>
    },
    SongList {
        list_of_songs: VecDeque<PlayableSong>
    },
    Success,
    Fail,
}

impl SongActor {
    fn new(receiver: mpsc::Receiver<SongActorMessage>, sse_broadcaster: Arc<sync::broadcast::Sender<PlayableSong>>) -> Self {
        SongActor {
            receiver: receiver,
            sse_broadcaster: sse_broadcaster,
            song_deque: VecDeque::new(),
            current_song: None,
        }
    }

    async fn handle_message(&mut self, msg: SongActorMessage) {
        match msg {
            SongActorMessage::QueueSong {
                playable_song,
                respond_to,
            } => {
                self.song_deque.push_back(playable_song);

                println!("SONG QUEUE: {:?}", self.song_deque);

                let _ = respond_to.send(SongActorResponse::Success);
            }
            SongActorMessage::RemoveSong {
                song_uuid,
                respond_to,
            } => {
                if let Some(index) = self.song_deque.iter().position(|x| (*x).uuid == song_uuid) {
                    self.song_deque.remove(index);
                }

                let _ = respond_to.send(SongActorResponse::Success);
            }
            SongActorMessage::PopSong { respond_to } => {
                let popped_song = self.song_deque.pop_front().map(|song| {
                    self.current_song = Some(song.clone());

                    let _ = self.sse_broadcaster.send(song.clone());

                    song
                });

                let _ = respond_to.send(SongActorResponse::CurrentSong { optional_song: popped_song });
            }
            SongActorMessage::Reposition {
                song_uuid,
                position,
                respond_to,
            } => {
                if let Some(current_index) =
                    self.song_deque.iter().position(|x| (*x).uuid == song_uuid)
                {
                    let song = self.song_deque.remove(current_index).unwrap();
                    let new_position = position.min(self.song_deque.len());
                    self.song_deque.insert(new_position, song);
                }

                let _ = respond_to.send(SongActorResponse::Success);
            }
            SongActorMessage::Current { respond_to } => {
                let _ = respond_to.send(SongActorResponse::CurrentSong { optional_song: self.current_song.clone() });
            }
        }
    }
}

async fn run_song_actor(mut actor: SongActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}

#[derive(Clone)]
pub struct SongActorHandle {
    sender: mpsc::Sender<SongActorMessage>,
}

impl SongActorHandle {
    pub fn new(sse_broadcaster: Arc<sync::broadcast::Sender<PlayableSong>>) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let song_actor = SongActor::new(receiver, sse_broadcaster);
        tokio::spawn(run_song_actor(song_actor));

        Self { sender }
    }

    pub async fn queue_song(&self, playable_song: PlayableSong) -> SongActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::QueueSong {
            playable_song: playable_song,
            respond_to: send,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn remove_song(&self, song_uuid: Uuid) -> SongActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::RemoveSong {
            song_uuid: song_uuid,
            respond_to: send,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn pop_song(&self) -> SongActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::PopSong { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn reposition_song(&self, song_uuid: Uuid, position: usize) -> SongActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::Reposition {
            song_uuid: song_uuid,
            position: position,
            respond_to: send,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn current_song(&self) -> SongActorResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::Current { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}
