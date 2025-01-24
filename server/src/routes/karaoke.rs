// use std::sync::{Arc, Mutex};

// use axum::{debug_handler, extract::{ Json, State }, http::StatusCode, response::{IntoResponse, Result, Response}};
// use serde::Deserialize;

// use crate::{actors::request::RequestActorHandle, queue::{ PlayableSong }, state::AppState, ytdlp::{Ytdlp, YtdlpError}};

use std::{collections::VecDeque, convert::Infallible, sync::Arc, time::Duration};

use axum::{
    body::Body, debug_handler, extract::{Path, State}, http::{header::{self, ACCEPT_RANGES}, HeaderMap, StatusCode}, response::{sse::{Event, KeepAlive}, IntoResponse, Response, Sse}, Json
};
use axum_extra::{headers, TypedHeader};
use futures_util::{stream, FutureExt, StreamExt};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use tokio::{fs::File, sync};
use tokio_util::io::ReaderStream;

use crate::{actors::{song_coordinator::{CurrentSongResponse, GetQueueResponse, PlayableSong, PopSongResponse, QueueSongResponse, QueueableSong, SongActorHandle}, video_downloader::{DownloadVideoResponse, VideoDlActorHandle}}, state::AppState, ytdlp::YtdlpError};

#[derive(Deserialize)]
pub struct QueueSong {
    yt_link: String,
}

impl IntoResponse for YtdlpError {
    fn into_response(self) -> axum::response::Response {
        return axum::response::Response::new("hi".into());
    }
}

#[debug_handler(state = AppState)]
pub async fn queue_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
    State(videodl_actor_handle): State<Arc<VideoDlActorHandle>>,
    Json(payload): Json<QueueSong>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("helo beanie 1");

    println!("ytlkink {}", payload.yt_link);
    
    let queueable_song = QueueableSong {
        name: String::from("helo beanie"),
        yt_link: payload.yt_link
    };

    let song_actor_handle_clone = Arc::clone(&song_actor_handle);
    tokio::spawn(async move {
        match videodl_actor_handle.download_video(queueable_song.yt_link).await {
            DownloadVideoResponse::Success { song_name, video_file_path } => {
                let queue_song_response = song_actor_handle_clone.queue_song(
                    PlayableSong::new(song_name, video_file_path)
                ).await;

                match queue_song_response {
                    QueueSongResponse::Success => {
                        println!("queued up the song");
                    }
                    QueueSongResponse::Fail => {
                        println!("wasn't able to queue up the song :(");
                    }
                }
            }
            DownloadVideoResponse::Fail => {
                println!("wasn't able to download the video :(");
            }
        }
    });


    Ok(StatusCode::ACCEPTED)
}

pub async fn play_next_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>
) -> Result<impl IntoResponse, StatusCode> {

    println!("helo beanie 3");

    let song_actor_response = song_actor_handle.pop_song().await;
    match song_actor_response {
        PopSongResponse::Success(next_song) => {
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
        queue: VecDeque<PlayableSong>
    },
    CurrentSongUpdated {
        current_song: Option<PlayableSong>
    }
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

    let video_codec = "copy";
    let video_bitrate = "5M";

    let audio_codec = "aac";

    
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