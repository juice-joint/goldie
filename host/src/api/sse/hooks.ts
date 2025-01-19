import { useQuery } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";
import type { CurrentSongChangeEvent, QueueUpdatedEvent } from "./types";

export const useCurrentSongChanges = () => {
  const queryData = useQuery<CurrentSongChangeEvent["current_song"]>({
    queryKey: QUERY_KEYS.currentSong,
    enabled: true,
  });
  return { ...queryData, data: { ...queryData.data } };
};

export const useQueueChanges = () => {
  const { data: queueData } = useQuery<QueueUpdatedEvent["queue"]>({
    queryKey: QUERY_KEYS.queue,
    enabled: true,
  });

  return queueData || [];
};
