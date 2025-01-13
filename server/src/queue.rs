use std::collections::VecDeque;

use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Clone)]
#[derive(Debug)] 
pub struct PlayableSong {
    pub name: String,
    pub uuid: Uuid
}

impl PlayableSong {
    pub fn new(name: &str) -> Self {
        PlayableSong {
            name: name.to_string(),
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

#[derive(Clone)]
pub struct SongQueue {
    song_deque: VecDeque<PlayableSong>,
    current_song: Option<PlayableSong>
}

impl SongQueue {
    pub fn new() -> Self {
        return Self {
            song_deque: VecDeque::new(),
            current_song: None
        }
    }

    pub fn queue(&mut self, song: PlayableSong) {
        self.song_deque.push_back(song);
    }

    pub fn remove(&mut self, song: PlayableSong) {
        if let Some(index) = self.song_deque.iter().position(|x| *x == song) {
            self.song_deque.remove(index);
        }
    }

    pub fn pop(&mut self) -> Option<PlayableSong> {
        self.song_deque.pop_front().map(|song| {
            self.current_song = Some(song.clone());
            song
        })
    }

    pub fn reposition(&mut self, song: PlayableSong, position: usize) {
        if let Some(current_index) = self.song_deque.iter().position(|x| *x == song) {
            let song = self.song_deque.remove(current_index).unwrap();
            let new_position = position.min(self.song_deque.len());
            self.song_deque.insert(new_position, song);
        }
    }

    pub fn current(&self) -> Option<&PlayableSong> {
        self.current_song.as_ref()
    }
}

struct SongActor {
    receiver: mpsc::Receiver<SongActorMessage>,
    song_deque: VecDeque<PlayableSong>,
    current_song: Option<PlayableSong>
}

pub enum SongActorMessage {
    QueueSong {
        respond_to: oneshot::Sender<SongActorResponse>
    },
    RemoveSong {
        respond_to: oneshot::Sender<SongActorResponse>
    },
    PopSong {
        respond_to: oneshot::Sender<SongActorResponse>
    },
    Reposition {
        respond_to: oneshot::Sender<SongActorResponse>
    },
    Current {
        respond_to: oneshot::Sender<SongActorResponse>
    }
}

pub enum SongActorResponse {
    CurrentSong(Option<PlayableSong>),
    Success,
    Fail
}

impl SongActor {
    fn new (
        receiver: mpsc::Receiver<SongActorMessage>,
    ) -> Self {
        SongActor {
            receiver: receiver,
            song_deque: VecDeque::new(),
            current_song: None
        }
    }

    async fn handle_message(&mut self, msg: SongActorMessage) {
        match msg {
            SongActorMessage::QueueSong { respond_to } => {

                let _ = respond_to.send(SongActorResponse::Success);    
            },
            SongActorMessage::RemoveSong { respond_to } => {
                let _ = respond_to.send(SongActorResponse::Success);
            },
            SongActorMessage::PopSong { respond_to } => {
                let _ = respond_to.send(SongActorResponse::Success);
            },
            SongActorMessage::Reposition { respond_to } => {
                let _ = respond_to.send(SongActorResponse::Success);
            },
            SongActorMessage::Current { respond_to } => {
                let _ = respond_to.send(SongActorResponse::CurrentSong(self.current_song.clone()));
            }
        }
    }
}

