use std::{collections::VecDeque, sync::Arc, usize};

use tokio::sync::{self, mpsc, oneshot};
use uuid::Uuid;

use crate::routes::karaoke::SseEvent;

#[derive(Clone, Debug, serde::Serialize)]
pub struct QueueableSong {
    pub name: String,
    pub yt_link: String
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct PlayableSong {
    pub name: String,
    pub video_file_path: String,
    #[serde(serialize_with = "serialize_uuid")]
    pub uuid: Uuid,
}

fn serialize_uuid<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(uuid.to_string().as_str())
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
    sse_broadcaster: Arc<sync::broadcast::Sender<SseEvent>>
}

pub enum SongActorMessage {
    QueueSong {
        playable_song: PlayableSong,
        respond_to: oneshot::Sender<QueueSongResponse>,
    },
    RemoveSong {
        song_uuid: Uuid,
        respond_to: oneshot::Sender<RemoveSongResponse>,
    },
    PopSong {
        respond_to: oneshot::Sender<PopSongResponse>,
    },
    Reposition {
        song_uuid: Uuid,
        position: usize,
        respond_to: oneshot::Sender<RepositionSongResponse>,
    },
    Current {
        respond_to: oneshot::Sender<CurrentSongResponse>,
    },
    GetQueue {
        respond_to: oneshot::Sender<GetQueueResponse>
    }
}

pub enum QueueSongResponse {
    Success,
    Fail
}

pub enum RemoveSongResponse {
    Success,
    Fail
}

pub enum PopSongResponse {
    Success(Option<PlayableSong>),
    Fail
}

pub enum RepositionSongResponse {
    Success,
    Fail
}

pub enum CurrentSongResponse {
    Success(Option<PlayableSong>),
    Fail
}

pub enum GetQueueResponse {
    Success(VecDeque<PlayableSong>),
    Fail
}

impl SongActor {
    fn new(receiver: mpsc::Receiver<SongActorMessage>, sse_broadcaster: Arc<sync::broadcast::Sender<SseEvent>>) -> Self {
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
                if self.song_deque.len() == 0 && self.current_song.is_none() {
                    self.current_song = Some(playable_song.clone());
                    println!("sent current song updated");
                    let _ = self.sse_broadcaster.send(SseEvent::CurrentSongUpdated { current_song: Some(playable_song.clone()) });
                } else {
                    self.song_deque.push_back(playable_song);
                    println!("sent queue updated");
                    let _ = self.sse_broadcaster.send(SseEvent::QueueUpdated { queue: self.song_deque.clone() });
                }

                let _ = respond_to.send(QueueSongResponse::Success);
            }
            SongActorMessage::RemoveSong {
                song_uuid,
                respond_to,
            } => {
                if let Some(index) = self.song_deque.iter().position(|x| (*x).uuid == song_uuid) {
                    self.song_deque.remove(index);
                }

                let _ = respond_to.send(RemoveSongResponse::Success);
            }
            SongActorMessage::PopSong { respond_to } => {
                let popped_song = self.song_deque.pop_front();

                self.current_song = popped_song.clone();
                let _ = self.sse_broadcaster.send(SseEvent::CurrentSongUpdated { current_song: popped_song.clone() });
                let _ = self.sse_broadcaster.send(SseEvent::QueueUpdated { queue: self.song_deque.clone() });
                let _ = respond_to.send(PopSongResponse::Success(popped_song));
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
                    let _ = self.sse_broadcaster.send(SseEvent::QueueUpdated { queue: self.song_deque.clone() });
                }

                let _ = respond_to.send(RepositionSongResponse::Success);
            }
            SongActorMessage::Current { respond_to } => {
                let _ = respond_to.send(CurrentSongResponse::Success(self.current_song.clone()));
            },
            SongActorMessage::GetQueue { respond_to } => {
                let _ = respond_to.send(GetQueueResponse::Success(self.song_deque.clone()));
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
    pub fn new(sse_broadcaster: Arc<sync::broadcast::Sender<SseEvent>>) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let song_actor = SongActor::new(receiver, sse_broadcaster);
        tokio::spawn(run_song_actor(song_actor));

        Self { sender }
    }

    pub async fn queue_song(&self, playable_song: PlayableSong) -> QueueSongResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::QueueSong {
            playable_song: playable_song,
            respond_to: send,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn remove_song(&self, song_uuid: Uuid) -> RemoveSongResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::RemoveSong {
            song_uuid: song_uuid,
            respond_to: send,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn pop_song(&self) -> PopSongResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::PopSong { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn reposition_song(&self, song_uuid: Uuid, position: usize) -> RepositionSongResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::Reposition {
            song_uuid: song_uuid,
            position: position,
            respond_to: send,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn current_song(&self) -> CurrentSongResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::Current { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
    
    pub async fn get_queue(&self) -> GetQueueResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::GetQueue { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}
