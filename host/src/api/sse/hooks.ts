import { useQuery } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";
import type { CurrentSongChangeEvent, QueueUpdatedEvent } from "./types";

export const useCurrentSong = () => {
  const { data: currentSong } = useQuery<
    CurrentSongChangeEvent["current_song"]
  >({
    queryKey: QUERY_KEYS.currentSong,
    enabled: true,
  });

  return currentSong || null;
};

export const useQueueChanges = () => {
  const { data: queueData } = useQuery<QueueUpdatedEvent["queue"]>({
    queryKey: QUERY_KEYS.queue,
    enabled: true,
  });

  return queueData || null;
};
