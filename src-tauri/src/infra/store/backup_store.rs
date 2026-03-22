/// 备份文件管理
use crate::domain::identity::BackupInfo;
use crate::error::AppError;
use std::path::PathBuf;

/// 备份文件存储
pub struct BackupStore {
    storage_json_dir: PathBuf,
}

impl BackupStore {
    pub fn new(storage_json_path: &PathBuf) -> Self {
        let dir = storage_json_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            storage_json_dir: dir,
        }
    }

    /// 查找所有备份文件
    pub fn find_backups(&self) -> Result<Vec<BackupInfo>, AppError> {
        let mut backups = Vec::new();

        if !self.storage_json_dir.exists() {
            return Ok(backups);
        }

        for entry in std::fs::read_dir(&self.storage_json_dir)? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();

            if (filename.contains(".bak.") || filename.contains(".backup."))
                && !filename.contains(".seamless")
            {
                let metadata = entry.metadata()?;
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or_default();

                backups.push(BackupInfo {
                    path: entry.path().to_string_lossy().to_string(),
                    filename,
                    timestamp: modified.clone(),
                    size: metadata.len(),
                    date_formatted: modified,
                });
            }
        }

        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(backups)
    }
}
