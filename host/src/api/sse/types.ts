import { Song } from "../api-types";

export enum EventType {
  QueueChangeEvent = "QueueUpdated",
  CurrentSongUpdated = "CurrentSongUpdated",
}

export type QueueUpdatedEvent = {
  type: "QueueUpdated";
  queue: Song[];
};

export type CurrentSongChangeEvent = {
  type: "CurrentSongUpdated";
  current_song: Song;
};

export type SSEEvent = QueueUpdatedEvent | CurrentSongChangeEvent;
