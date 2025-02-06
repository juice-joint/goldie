import { useQuery } from "@tanstack/react-query";
import axiosClient from "../axios";
import { QUERY_KEYS } from "../queryKeys";
import { FormattedSong, Song } from "../api-types";
import { formatSong } from "../../utils/format";

async function getCurrentSong(): Promise<FormattedSong | null> {
  const { data: song, status } = await axiosClient.get<Song>("/current_song", {
    headers: { "Content-Type": "application/json", Accept: "*" },
  });
  if (status === 204) {
    return null;
  }
  console.log(song);

  const formattedSong = formatSong(song);
  return formattedSong;
}

export function useCurrentSong() {
  return useQuery({
    queryFn: getCurrentSong,
    queryKey: QUERY_KEYS.currentSong,
    enabled: true,
  });
}
