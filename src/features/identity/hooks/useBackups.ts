/** 备份管理 hooks（TanStack Query） */
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

/** 查询备份列表 */
export function useBackups() {
  return useQuery({
    queryKey: ["backups"],
    queryFn: () => invoke("get_available_backups"),
  });
}

/** 从备份恢复 Machine ID */
export function useRestoreBackup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (backupPath: string) =>
      invoke("restore_machine_ids", { backupPath }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["machine-ids"] });
    },
  });
}

/** 从备份提取 ID */
export function useExtractBackupIds() {
  return useMutation({
    mutationFn: (backupPath: string) =>
      invoke("extract_backup_ids", { backupPath }),
  });
}

/** 删除备份 */
export function useDeleteBackup() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (backupPath: string) =>
      invoke("delete_backup", { backupPath }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["backups"] });
    },
  });
}
