/** Machine ID 操作 hooks（TanStack Query） */
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

/** 查询当前 Machine ID */
export function useCurrentIds() {
  return useQuery({
    queryKey: ["machine-ids"],
    queryFn: () => invoke("get_current_machine_ids"),
  });
}

/** 重置 Machine ID */
export function useResetIds() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => invoke("reset_machine_ids"),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["machine-ids"] });
      qc.invalidateQueries({ queryKey: ["backups"] });
    },
  });
}

/** 完全重置（ID + main.js + workbench.js） */
export function useCompleteReset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => invoke("complete_cursor_reset"),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["machine-ids"] });
    },
  });
}

/** 获取 Cursor 路径信息 */
export function useCursorPaths() {
  return useQuery({
    queryKey: ["cursor-paths"],
    queryFn: () => invoke<[string, string]>("get_cursor_paths"),
  });
}

/** 检查 Cursor 安装状态 */
export function useCursorInstallation() {
  return useQuery({
    queryKey: ["cursor-installation"],
    queryFn: () => invoke<boolean>("check_cursor_installation"),
  });
}

/** 获取 machineId 文件内容 */
export function useMachineIdFileContent() {
  return useQuery({
    queryKey: ["machine-id-file"],
    queryFn: () => invoke<string | null>("get_machine_id_file_content"),
  });
}
