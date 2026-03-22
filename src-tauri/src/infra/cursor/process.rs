/// Cursor 进程检测与管理
use crate::error::AppError;

/// 进程管理器
pub struct ProcessManager;

impl ProcessManager {
    pub fn new() -> Self {
        Self
    }

    /// 确保 Cursor 未在运行（写入操作前调用）
    pub fn ensure_not_running(&self) -> Result<(), AppError> {
        if self.is_running() {
            return Err(AppError::CursorRunning);
        }
        Ok(())
    }

    /// 检查 Cursor 是否正在运行
    pub fn is_running(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            Self::check_process_windows("Cursor.exe")
        }

        #[cfg(target_os = "macos")]
        {
            Self::check_process_unix("Cursor")
        }

        #[cfg(target_os = "linux")]
        {
            Self::check_process_unix("cursor")
        }
    }

    #[cfg(target_os = "windows")]
    fn check_process_windows(name: &str) -> bool {
        std::process::Command::new("tasklist")
            .args(&["/FI", &format!("IMAGENAME eq {}", name)])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(name))
            .unwrap_or(false)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn check_process_unix(name: &str) -> bool {
        std::process::Command::new("pgrep")
            .args(&["-x", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
