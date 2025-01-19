import { useQuery } from "@tanstack/react-query";
import axiosClient from "../axios";
import { QUERY_KEYS } from "../queryKeys";

async function getQueue() {
  const { data } = await axiosClient.get("/song_list", {
    headers: { "Content-Type": "application/json", Accept: "*" },
  });

  return data;
}

export function useQueue() {
  return useQuery({
    queryFn: getQueue,
    queryKey: QUERY_KEYS.queue,
  });
}
