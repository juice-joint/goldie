import { useQuery } from "@tanstack/react-query";
import axiosClient from "../axios";
import { QUERY_KEYS } from "../queryKeys";

async function getCurrentSong() {
  const { data, status } = await axiosClient.get("/current_song", {
    headers: { "Content-Type": "application/json", Accept: "*" },
  });
  if (status === 204) {
    return null;
  }
  return {
    ...data,
  };
}

export function useCurrentSong() {
  return useQuery({
    queryFn: getCurrentSong,
    queryKey: QUERY_KEYS.currentSong,
    enabled: true,
  });
}
