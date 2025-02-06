import { useQuery } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";

export const useKey = () => {
  const { data: key } = useQuery<number>({
    queryKey: QUERY_KEYS.key,
    enabled: true,
  });

  return key ?? 0;
};
