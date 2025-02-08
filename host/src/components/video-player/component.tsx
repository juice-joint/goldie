import { useCallback, useEffect, useRef, useState } from "react";
import { usePlayNextSong } from "../../api/mutations/usePlayNextSong";
import queryClient from "../../api/queryClient";
import { QUERY_KEYS } from "../../api/queryKeys";
import { API_URL } from "../../api/sse/eventSource";
import { useCurrentSong } from "../../api/queries/useCurrentSong";
import dashjs from "dashjs";
import { usePlayback } from "../../api/queries/usePlayback";
import { useKey } from "../../api/queries/useKey";
import { Status } from "../../api/api-types";

function VideoPlayer() {
  const currentSong = useCurrentSong();
  const vidRef = useRef<HTMLVideoElement>(null);
  const playerRef = useRef<dashjs.MediaPlayerClass | null>(null);
  const { mutate: playNextSong } = usePlayNextSong();
  const [progress, setProgress] = useState(0);
  const playbackState = usePlayback();
  const key = useKey();

  const handleEnded = useCallback(() => {
    queryClient.setQueryData(QUERY_KEYS.currentSong, null);
    playNextSong();
  }, [playNextSong]);

  const handleError = useCallback((e: any) => {
    console.log("error", e);
  }, []);

  const switchToTrack = useCallback(
    (player: dashjs.MediaPlayerClass, trackId: string) => {
      const tracks = player.getTracksFor("audio");
      console.log(tracks);
      console.log(trackId);

      const targetTrack = tracks.find(
        (track) => track.id?.toString() === trackId
      );
      if (targetTrack) {
        player.setCurrentTrack(targetTrack);
        console.log("switched to track", trackId);
      }
    },
    []
  );

  useEffect(() => {
    if (playbackState) {
      playerRef.current?.play();
    } else {
      playerRef.current?.pause();
    }
  }, [playbackState]);

  useEffect(() => {
    console.log(key);

    if (playerRef.current) {
      console.log("hello");

      switchToTrack(playerRef.current, (key + 4).toString());
    }
  }, [key, switchToTrack, currentSong, playerRef.current]);

  useEffect(() => {
    if (
      currentSong?.name &&
      vidRef.current &&
      currentSong.status === Status.Success
    ) {
      // destroy existing player if it exists
      if (playerRef.current) {
        playerRef.current.destroy();
      }

      // initialize dash.js player
      const player = dashjs.MediaPlayer().create();
      playerRef.current = player;
      console.log(currentSong.name);
      // TODO: the first segment is the lowest key.
      player.initialize(
        vidRef.current,
        `${API_URL}/dash/${currentSong.name}/${currentSong.name}.mpd`,
        true
      );
      player.on(dashjs.MediaPlayer.events.PLAYBACK_ENDED, handleEnded);
      player.on(
        dashjs.MediaPlayer.events.PLAYBACK_TIME_UPDATED,
        handleTimeUpdate
      );

      player.on(dashjs.MediaPlayer.events.ERROR, handleError);

      player.setQualityFor("audio", 2, true); // 1 should be the high quality representation, 0 would be normal
      // player.setInitialMediaSettingsFor("audio", { role: "main" });
      // configure quality and segment template
      player.updateSettings({
        streaming: {
          abr: {
            autoSwitchBitrate: { video: false, audio: false },
          },
          buffer: {
            fastSwitchEnabled: true,
            stallThreshold: 0.5,
            bufferTimeAtTopQuality: 30,
            bufferToKeep: 30,
            bufferPruningInterval: 30,
            stableBufferTime: 5,
          },
          scheduling: {
            scheduleWhilePaused: true,
          },
        },
      });
    }

    return () => {
      if (playerRef.current) {
        playerRef.current.destroy();
        playerRef.current = null;
      }
    };
  }, [currentSong, handleEnded, handleError, switchToTrack]);

  const handleTimeUpdate = () => {
    if (playerRef.current) {
      const duration = playerRef.current.duration();
      const currentTime = playerRef.current.time();
      if (duration > 0) {
        setProgress((currentTime / duration) * 100);
      }
    }
  };

  if (!currentSong?.name) {
    return null;
  }

  console.log(currentSong);

  return (
    <div className="relative w-full h-full bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900">
      {currentSong.status === Status.InProgress && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-black/40 backdrop-blur-sm">
          <div className="w-16 h-16 border-4 border-purple-200 border-t-purple-500 rounded-full animate-spin" />
          <div className="mt-4 text-center">
            <p className="text-white text-lg font-medium mt-1">
              downloading {currentSong.formattedName}...
            </p>
          </div>
        </div>
      )}
      {currentSong.status === Status.Success && (
        <video
          className="w-full h-full rounded-lg shadow-2xl"
          ref={vidRef}
          controls
        />
      )}
      <div className="absolute bottom-0 left-0 right-0">
        <div className="h-1 bg-gray-800/50">
          <div
            className="h-full bg-purple-500 transition-all duration-300 ease-linear"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>
    </div>
  );
}

export default VideoPlayer;
