use tokio::sync::{mpsc, oneshot};

use crate::actors::videodl::VideoDlActorHandle;

struct RequestActor {
    receiver: mpsc::Receiver<RequestActorMessage>,
    videodl_actor_handle: VideoDlActorHandle,
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
    List {
        respond_to: oneshot::Sender<RequestActorResponse>
    }
}

#[derive(Debug)]
pub enum RequestActorResponse {
    Success,
    Fail
}

impl RequestActor {
    fn new(
        receiver: mpsc::Receiver<RequestActorMessage>, 
        videodl_actor_handle: VideoDlActorHandle
    ) -> Self {
        RequestActor {
            receiver: receiver,
            videodl_actor_handle: videodl_actor_handle
        }
    }

    async fn handle_message(&mut self, msg: RequestActorMessage) {
        match msg {
            RequestActorMessage::QueueSong { respond_to } => {
                self.videodl_actor_handle.download_video(String::from("test")).await;

                let _ = respond_to.send(RequestActorResponse::Success);
            }
            RequestActorMessage::PlayNext { respond_to } => todo!(),
            RequestActorMessage::Remove { respond_to } => todo!(),
            RequestActorMessage::ReOrder { respond_to } => todo!(),
            RequestActorMessage::List { respond_to } => todo!(),
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
    pub fn new(videodl_actor_handle: VideoDlActorHandle) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let request_actor = RequestActor::new(receiver, videodl_actor_handle);
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
}