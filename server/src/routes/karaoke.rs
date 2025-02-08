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

use crate::{
    actors::{
        song_coordinator::{
            QueuedSongStatus, Song, SongActorHandle, SongCoordinatorError
        },
        video_downloader::{DownloadVideoResponse, VideoDlActorHandle},
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
) -> Result<impl IntoResponse, StatusCode> {
    println!("helo beanie 1");

    let queueable_song = Song::new(payload.name, payload.yt_link, QueuedSongStatus::InProgress);

    println!("queuesongreuqest {:?}", queueable_song);
    // TODO fix this to use references

    match song_actor_handle.queue_song(queueable_song.clone()).await {
        Ok(_) => {
            tokio::spawn(async move {
                match videodl_actor_handle
                    .download_video(queueable_song.yt_link, queueable_song.name.to_string())
                    .await
                {
                    DownloadVideoResponse::Success { video_file_path } => {
                        println!("receieved downloaded video file path");
                        match song_actor_handle
                            .update_song_status(queueable_song.uuid, QueuedSongStatus::Success)
                            .await
                        {
                            Ok(_) => {
                                println!("updated queued song status!");

                                // match song_actor_handle.initialize(queueable_song.uuid).await {
                                //     InitializeResponse::Success => {
                                //         println!("updated current song initializaton");
                                //     }
                                //     InitializeResponse::Fail => {
                                //         println!("could not update current song initialization");
                                //     }
                                // }
                            }
                            Err(_) => {
                                println!("wasn't able to update queued song status :(");
                                // TODO deal with failed, should pop songs until success
                            }
                        }

                        std::fs::remove_file(&video_file_path).unwrap_or_else(|err| {
                            println!("unable to delete file {} with error: {}", &video_file_path, err);
                        });
                    }
                    DownloadVideoResponse::Fail => {
                        println!("wasn't able to download the video :(");

                        match song_actor_handle
                            .update_song_status(queueable_song.uuid, QueuedSongStatus::Failed)
                            .await
                        {
                            Ok(_) => {
                                println!("updated queued song status!");
                            }
                            Err(_) => {
                                println!("wasn't able to update queued song status :(");
                            }
                        }
                    }
                }
            });
        }
        Err(_) => {
            println!("wasn't able to queue up the song :(");
        }
    }

    Ok(StatusCode::ACCEPTED)
}

pub async fn play_next_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("helo beanie 3");

    let song_actor_response = song_actor_handle.pop_song().await;
    match song_actor_response {
        Ok(_popped_song) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn song_list(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {
    let song_actor_response = song_actor_handle.get_queue().await;
    match song_actor_response {
        Ok(list_of_songs) => Ok((StatusCode::OK, Json(list_of_songs))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn current_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {
    let song_actor_response = song_actor_handle.current_song().await;
    match song_actor_response {
        Ok(current_song) => match current_song {
            Some(current_song) => Ok((StatusCode::OK, Json(current_song))),
            None => Err(StatusCode::NO_CONTENT),
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
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

pub async fn here_video(Path(video): Path<String>) -> Result<Response<Body>, StatusCode> {
    // Open the file

    let file = File::open(format!("assets/{}.mp4", video))
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Create a stream from the file
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Build the response with appropriate headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4") // Adjust content type as needed
        .header(ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}
