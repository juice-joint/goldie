use std::{collections::VecDeque, convert::Infallible, sync::Arc};

use axum::{
    body::Body, debug_handler, extract::{Path, State}, http::{header::{self, ACCEPT_RANGES}, StatusCode}, response::{sse::{Event, KeepAlive}, IntoResponse, Response, Sse}, Json
};
use futures_util::{stream, StreamExt};
use serde::Deserialize;
use tokio::{fs::File, sync};
use tokio_util::io::ReaderStream;

use crate::{actors::{song_coordinator::{CurrentSongResponse, GetQueueResponse, PopSongResponse, QueueSongResponse, QueuedSongStatus, Song, SongActorHandle, UpdateSongStatusResponse}, video_downloader::{DownloadVideoResponse, VideoDlActorHandle}}, lib::yt_downloader::VideoProcessError, state::AppState};

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
    
    let song_uuid = queueable_song.uuid.clone();
    let yt_link = queueable_song.yt_link.clone();

    let queue_song_response = song_actor_handle.queue_song(queueable_song).await;
    match queue_song_response {
        QueueSongResponse::Success => {
            tokio::spawn(async move {
                match videodl_actor_handle.download_video(yt_link).await {
                    DownloadVideoResponse::Success { video_file_path } => {
                        println!("receieved downloaded video file path");
                        
                          // First split by the last '/'
                        // let name = video_file_path.rsplit_once('/')
                        //     .map(|(_, name)| name.to_string())
                        //     .ok_or_else(|| "invalid");
        
                        let update_song_response = song_actor_handle.update_song_status(
                            song_uuid, QueuedSongStatus::Success
                        ).await;
                        match update_song_response {
                            UpdateSongStatusResponse::Success => {
                                println!("updated queued song status!");
                            }
                            UpdateSongStatusResponse::Fail => {
                                println!("wasn't able to update queued song status :(");
                            }
                        }

                    }
                    DownloadVideoResponse::Fail => {
                        println!("wasn't able to download the video :(");
                    }
                }
            });
        }
        QueueSongResponse::Fail => {
            println!("wasn't able to queue up the song :(");   
        }
    }


    

    Ok(StatusCode::ACCEPTED)
}

pub async fn play_next_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>
) -> Result<impl IntoResponse, StatusCode> {

    println!("helo beanie 3");

    let song_actor_response = song_actor_handle.pop_song().await;
    match song_actor_response {
        PopSongResponse::Success(_) => {
            Ok(StatusCode::OK)
        },
        PopSongResponse::Fail => {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn song_list(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {

    let song_actor_response = song_actor_handle.get_queue().await;
    match song_actor_response {
        GetQueueResponse::Success(list_of_songs) => {
            Ok((StatusCode::OK, Json(list_of_songs)))
        },
        GetQueueResponse::Fail => {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn current_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {

    let song_actor_response = song_actor_handle.current_song().await;
    match song_actor_response {
        CurrentSongResponse::Success(current_song ) => {
            match current_song {
                Some(current_song) => {
                    Ok((StatusCode::OK, Json(current_song)))
                },
                None => {
                    Err(StatusCode::NO_CONTENT)
                }
            }
        }
        CurrentSongResponse::Fail => {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum SseEvent {
    QueueUpdated {
        queue: VecDeque<Song>
    },
    CurrentSongUpdated {
        current_song: Option<Song>
    },
    KeyChange {
        current_key: i8
    },
    TogglePlayback
}

pub async fn sse(
    State(sse_broadcaster): State<Arc<sync::broadcast::Sender<SseEvent>>>
) -> Sse<impl stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::BroadcastStream::new(sse_broadcaster.subscribe())
        .filter_map(|result| async move {
            match result {
                Ok(sse_event) => {
                    let event_json = serde_json::to_string(&sse_event).ok()?;
                    Some(Ok(Event::default().data(event_json)))
                },
                Err(_) => None
            }
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn here_video(
    Path(video): Path<String>
) -> Result<Response<Body>, StatusCode> {
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

