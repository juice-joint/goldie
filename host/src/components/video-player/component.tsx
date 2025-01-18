import { useRef, useState } from "react";
import { usePlayNextSong } from "../../api/mutations/usePlayNextSong";
import queryClient from "../../api/queryClient";
import { useCurrentSong } from "../../api/sse/hooks";
import { QUERY_KEYS } from "../../api/queryKeys";
import { API_URL } from "../../api/sse/eventSource";

function VideoPlayer() {
  const currentSong = useCurrentSong();

  const vidRef = useRef<HTMLVideoElement>(null);
  const { mutate: playNextSong } = usePlayNextSong();
  const [progress, setProgress] = useState(0);

  const handleEnded = () => {
    queryClient.setQueryData(QUERY_KEYS.currentSong, null);
    playNextSong();
  };

  const handleTimeUpdate = () => {
    if (vidRef.current) {
      const progress =
        (vidRef.current.currentTime / vidRef.current.duration) * 100;
      setProgress(progress);
    }
  };

  const videoUrl = `${API_URL}/${currentSong?.video_file_path}`;

  return (
    <div className="relative w-full h-full bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900">
      <video
        className="w-full h-full rounded-lg shadow-2xl"
        ref={vidRef}
        controls
        autoPlay
        onEnded={handleEnded}
        onTimeUpdate={handleTimeUpdate}
        muted
      >
        <source src={videoUrl} type="video/mp4" />
      </video>
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
