import { EventType } from "./sse/types";

export const QUERY_KEYS = {
  playNextSong: ["playNextSong"] as const,
  currentSong: [EventType.CurrentSongUpdated] as const,
  queue: ["sse", EventType.QueueChangeEvent] as const,
};
