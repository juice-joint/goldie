use std::{collections::VecDeque, sync::Arc, usize};

use tokio::sync::{self, mpsc, oneshot};
use uuid::Uuid;

use crate::routes::karaoke::SseEvent;

#[derive(Clone, Debug, serde::Serialize)]
pub enum QueuedSongStatus {
    InProgress,
    Failed,
    Success,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Song {
    pub name: String,
    #[serde(serialize_with = "serialize_uuid")]
    pub uuid: Uuid,
    pub yt_link: String,
    pub status: QueuedSongStatus
}

fn serialize_uuid<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(uuid.to_string().as_str())
}

impl Song {
    pub fn new(name: String, yt_link: String, status: QueuedSongStatus) -> Self {
        Song {
            name: name.to_string(),
            uuid: Uuid::new_v4(),
            yt_link: yt_link,
            status: status
        }
    }
}

impl ToString for Song {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl PartialEq for Song {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

struct SongActor {
    receiver: mpsc::Receiver<SongActorMessage>,
    song_deque: VecDeque<Song>,
    current_song: Option<Song>,
    current_key: i8,
    sse_broadcaster: Arc<sync::broadcast::Sender<SseEvent>>,
}

pub enum SongActorMessage {
    QueueSong {
        song: Song,
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
        respond_to: oneshot::Sender<GetQueueResponse>,
    },
    KeyUp {
        respond_to: oneshot::Sender<KeyUpResponse>,
    },
    KeyDown {
        respond_to: oneshot::Sender<KeyDownResponse>,
    },
    UpdateSongStatus {
        song_uuid: Uuid,
        status: QueuedSongStatus,
        respond_to: oneshot::Sender<UpdateSongStatusResponse>
    }
}

pub enum QueueSongResponse {
    Success,
    Fail,
}

pub enum RemoveSongResponse {
    Success,
    Fail,
}

pub enum PopSongResponse {
    Success(Option<Song>),
    Fail,
}

pub enum RepositionSongResponse {
    Success,
    Fail,
}

pub enum CurrentSongResponse {
    Success(Option<Song>),
    Fail,
}

pub enum GetQueueResponse {
    Success(VecDeque<Song>),
    Fail,
}

pub enum KeyUpResponse {
    Success(i8),
    Fail,
}

pub enum KeyDownResponse {
    Success(i8),
    Fail,
}

pub enum UpdateSongStatusResponse {
    Success,
    Fail
}

impl SongActor {
    fn new(
        receiver: mpsc::Receiver<SongActorMessage>,
        sse_broadcaster: Arc<sync::broadcast::Sender<SseEvent>>,
    ) -> Self {
        SongActor {
            receiver: receiver,
            sse_broadcaster: sse_broadcaster,
            song_deque: VecDeque::new(),
            current_song: None,
            current_key: 0,
        }
    }

    async fn handle_message(&mut self, msg: SongActorMessage) {
        match msg {
            SongActorMessage::QueueSong {
                song,
                respond_to,
            } => {
                self.song_deque.push_back(song);

                let _ = self.sse_broadcaster.send(SseEvent::QueueUpdated {
                    queue: self.song_deque.clone(),
                });

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
                let _ = self.sse_broadcaster.send(SseEvent::CurrentSongUpdated {
                    current_song: popped_song.clone(),
                });
                let _ = self.sse_broadcaster.send(SseEvent::QueueUpdated {
                    queue: self.song_deque.clone(),
                });
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
                    let _ = self.sse_broadcaster.send(SseEvent::QueueUpdated {
                        queue: self.song_deque.clone(),
                    });
                }

                let _ = respond_to.send(RepositionSongResponse::Success);
            }
            SongActorMessage::Current { respond_to } => {
                let _ = respond_to.send(CurrentSongResponse::Success(self.current_song.clone()));
            }
            SongActorMessage::GetQueue { respond_to } => {
                let _ = respond_to.send(GetQueueResponse::Success(self.song_deque.clone()));
            }
            SongActorMessage::KeyUp { respond_to } => {
                if self.current_key >= 3 {
                    // TODO fix this and grab it from some settings descriptor
                    let _ = respond_to.send(KeyUpResponse::Fail);
                } else {
                    self.current_key += 1;
                    let _ = self.sse_broadcaster.send(SseEvent::KeyChange {
                        current_key: self.current_key,
                    });

                    let _ = respond_to.send(KeyUpResponse::Success(self.current_key));
                }
            }
            SongActorMessage::KeyDown { respond_to } => {
                if self.current_key <= -3 {
                    // TODO fix this and grab it from some settings descriptor
                    let _ = respond_to.send(KeyDownResponse::Fail);
                } else {
                    self.current_key -= 1;
                    let _ = self.sse_broadcaster.send(SseEvent::KeyChange {
                        current_key: self.current_key,
                    });

                    let _ = respond_to.send(KeyDownResponse::Success(self.current_key));
                }
            },
            SongActorMessage::UpdateSongStatus { song_uuid, status, respond_to } => {
                if let Some(song) = self.song_deque.iter_mut().find(|song| song.uuid == song_uuid) {
                    song.status = status;
     
                    let _ = self.sse_broadcaster.send(SseEvent::QueueUpdated {
                        queue: self.song_deque.clone(),
                    });

                    let _ = respond_to.send(UpdateSongStatusResponse::Success);
                } else {
                    let _ = respond_to.send(UpdateSongStatusResponse::Fail);
                }
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

    pub async fn queue_song(&self, song: Song) -> QueueSongResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::QueueSong {
            song: song,
            respond_to: send,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn update_song_status(&self, song_uuid: Uuid, new_status: QueuedSongStatus) -> UpdateSongStatusResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::UpdateSongStatus {
            song_uuid: song_uuid,
            status: new_status,
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

    pub async fn reposition_song(
        &self,
        song_uuid: Uuid,
        position: usize,
    ) -> RepositionSongResponse {
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

    pub async fn key_up(&self) -> KeyUpResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::KeyUp { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn key_down(&self) -> KeyDownResponse {
        let (send, recv) = oneshot::channel();
        let msg = SongActorMessage::KeyDown { respond_to: send };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}
