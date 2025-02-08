import { useCurrentSong } from "../../api/queries/useCurrentSong";
import { useQueueChanges } from "../../api/sse/hooks";

export const Queue = () => {
  const queue = useQueueChanges();
  const currentSong = useCurrentSong();

  const nextSongs = queue?.slice(0, 2) || [];

  return (
    <div className="absolute bottom-4 right-4 w-64 bg-white rounded-lg shadow-xl overflow-hidden">
      {!!currentSong && (
        <div className="px-3 py-2 bg-purple-50 border-b border-purple-100">
          <div className="flex items-center gap-3">
            <div className="w-4 h-4 flex items-center justify-center rounded-full bg-purple-200">
              <div className="w-1.5 h-1.5 rounded-full bg-purple-500 animate-pulse" />
            </div>
            <p className="text-purple-900 text-xs font-medium truncate">
              {currentSong?.formattedName}
            </p>
          </div>
        </div>
      )}
      <div className="p-3 bg-purple-900">
        <h3 className="text-white text-sm font-medium uppercase tracking-wider">
          up next
        </h3>
      </div>
      <div>
        {nextSongs.length > 0 ? (
          nextSongs.map((song) => (
            <div
              key={song.uuid}
              className="p-3 border-gray-100 hover:bg-purple-50 transition-colors"
            >
              <div className="flex items-center gap-3">
                <div className="flex-1 min-w-0">
                  <p className="text-gray-900 text-sm truncate">
                    {song.formattedName}
                  </p>
                </div>
              </div>
            </div>
          ))
        ) : (
          <div className="p-3 border-gray-100">
            <div className="flex items-center gap-3">
              <div className="flex-1 min-w-0">
                <p className="text-gray-500 text-sm">None</p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
