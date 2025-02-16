use std::{collections::VecDeque, convert::Infallible, sync::Arc};

use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::{
        header::{self, ACCEPT_RANGES},
        StatusCode,
    },
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Response, Sse,
    },
    Json,
};
use futures_util::{stream, StreamExt};
use serde::Deserialize;
use tokio::{fs::File, sync};
use tokio_util::io::ReaderStream;
use tracing::{error, info, trace};

use crate::{
    actors::{
        song_coordinator::{QueuedSongStatus, Song, SongActorHandle, SongCoordinatorError},
        video_downloader::{VideoDlActorHandle},
    },
    state::AppState,
};

#[derive(Deserialize)]
pub struct QueueSong {
    name: String,
    yt_link: String,
}

#[debug_handler(state = AppState)]
pub async fn queue_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
    State(videodl_actor_handle): State<Arc<VideoDlActorHandle>>,
    Json(payload): Json<QueueSong>,
) -> impl IntoResponse {
    let queueable_song = Song::new(payload.name, payload.yt_link, QueuedSongStatus::InProgress);
    info!("received queue_song request: {}", queueable_song);

    match song_actor_handle.queue_song(queueable_song.clone()).await {
        Ok(_) => {
            info!("successfully queued song: {}", queueable_song.uuid);

            tokio::spawn(async move {
                match videodl_actor_handle
                    .download_video(queueable_song.yt_link, queueable_song.name.to_string())
                    .await
                {
                    Ok(video_file_path) => {
                        info!("successfully downloaded video in: {}", video_file_path);

                        match song_actor_handle
                            .update_song_status(queueable_song.uuid, QueuedSongStatus::Success)
                            .await
                        {
                            Ok(_) => {
                                info!(
                                    "successfully updated song: {} with status: {}",
                                    queueable_song.uuid,
                                    QueuedSongStatus::Success
                                );
                            }
                            Err(err) => {
                                error!(
                                    "unable to update status for song: {} with error: {}",
                                    queueable_song.uuid, err
                                );
                            }
                        }

                        std::fs::remove_file(&video_file_path).unwrap_or_else(|err| {
                            error!(
                                "unable to delete file {} with error: {}",
                                &video_file_path, err
                            );
                        });
                    }
                    Err(err) => {
                        error!(
                            "could not download video for song: {} with error: {}",
                            queueable_song.uuid, err
                        );

                        match song_actor_handle
                            .update_song_status(queueable_song.uuid, QueuedSongStatus::Failed)
                            .await
                        {
                            Ok(_) => {
                                info!(
                                    "successfully updated song: {} with status: {}",
                                    queueable_song.uuid,
                                    QueuedSongStatus::Failed
                                );
                            }
                            Err(err) => {
                                error!(
                                    "unable to update status for song: {} with error: {}",
                                    queueable_song.uuid, err
                                );
                            }
                        }
                    }
                }
            });
        }
        Err(err) => {
            error!(
                "unable to queue song: {} with error: {}",
                queueable_song.uuid, err
            );
        }
    }

    StatusCode::ACCEPTED
}

pub async fn play_next_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> impl IntoResponse {
    info!("received play_next_song request");

    match song_actor_handle.pop_song().await {
        Some(song) => {
            info!("successfully popped song: {}", song);
            StatusCode::OK
        }
        None => {
            info!("successfully popped song: {}", "none");
            StatusCode::OK
        }
    }
}

pub async fn song_list(State(song_actor_handle): State<Arc<SongActorHandle>>) -> impl IntoResponse {
    match song_actor_handle.get_queue().await {
        Ok(list_of_songs) => (StatusCode::OK, Json(list_of_songs)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn current_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> impl IntoResponse {
    let song_actor_response = song_actor_handle.current_song().await;
    match song_actor_response {
        Ok(current_song) => match current_song {
            Some(current_song) => (StatusCode::OK, Json(current_song)).into_response(),
            None => StatusCode::NO_CONTENT.into_response(),
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum SseEvent {
    QueueUpdated { queue: VecDeque<Song> },
    KeyChange { current_key: i8 },
    TogglePlayback,
}

pub async fn sse(
    State(sse_broadcaster): State<Arc<sync::broadcast::Sender<SseEvent>>>,
) -> Sse<impl stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::BroadcastStream::new(sse_broadcaster.subscribe())
        .filter_map(|result| async move {
            match result {
                Ok(sse_event) => {
                    let event_json = serde_json::to_string(&sse_event).ok()?;
                    Some(Ok(Event::default().data(event_json)))
                }
                Err(_) => None,
            }
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
