import { useQuery } from "@tanstack/react-query";
import axiosClient from "../axios";
import { QUERY_KEYS } from "../queryKeys";
import { Song } from "../api-types";
import { formatSong } from "../../utils/format";

async function getCurrentSong(): Promise<Song> {
  return { name: "test", video_file_path: "sample", uuid: "1" };
  // const { data: song, status } = await axiosClient.get<Song>("/current_song", {
  //   headers: { "Content-Type": "application/json", Accept: "*" },
  // });
  // if (status === 204) {
  //   return null;
  // }

  // const formattedSong = formatSong(song);
  // return formattedSong;
}

export function useCurrentSong() {
  return useQuery({
    queryFn: getCurrentSong,
    queryKey: QUERY_KEYS.currentSong,
    enabled: true,
  });
}
