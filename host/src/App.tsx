import { useCurrentSong } from "./api/sse/hooks";
import { useEventSource } from "./api/sse/useEventSource";
import { ErrorScreen } from "./components/error/component";
import QRCodeBanner from "./components/qr-code/component";
import { Queue } from "./components/queue/component";
import { Splash } from "./components/splash/component";
import { VideoPlayer } from "./components/video-player";
function App() {
  const { error, isLoading: isSSELoading } = useEventSource();

  const currentSong = useCurrentSong();

  if (isSSELoading) {
    return <Splash />;
  }

  if (error) {
    return <ErrorScreen />;
  }

  return (
    <div className="w-full h-full">
      {!currentSong && <Splash />}
      {currentSong && <VideoPlayer />}
      <QRCodeBanner />
      <Queue />
    </div>
  );
}

export default App;
