/** 用量数据查询 hooks */
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

/** 按时间段查询聚合用量数据 */
export function useAggregatedUsage(
  sessionToken: string | null,
  startDate: string,
  endDate: string
) {
  return useQuery({
    queryKey: ["usage", "aggregated", sessionToken, startDate, endDate],
    queryFn: () =>
      invoke("get_usage_for_period", { sessionToken, startDate, endDate }),
    enabled: !!sessionToken,
    staleTime: 5 * 60_000,
  });
}

/** 查询用户分析数据 */
export function useUserAnalytics(
  sessionToken: string | null,
  startDate: string,
  endDate: string
) {
  return useQuery({
    queryKey: ["usage", "analytics", sessionToken, startDate, endDate],
    queryFn: () =>
      invoke("get_user_analytics", { sessionToken, startDate, endDate }),
    enabled: !!sessionToken,
  });
}
