/// Machine ID 相关命令入口
///
/// 使用新架构 services::IdentityService 处理所有业务逻辑。
use crate::domain::identity::*;
use crate::services::identity_service::IdentityService;
use crate::{log_info, log_error};
use tauri::State;

/// 获取当前 Machine ID
#[tauri::command]
#[specta::specta]
pub async fn get_current_machine_ids(
    service: State<'_, IdentityService>,
) -> Result<Option<MachineIds>, String> {
    service.read_current()
        .map(|ids| Some(ids))
        .map_err(|e| e.to_string())
}

/// 获取可用备份列表
#[tauri::command]
#[specta::specta]
pub async fn get_available_backups(
    service: State<'_, IdentityService>,
) -> Result<Vec<BackupInfo>, String> {
    service.list_backups()
        .map_err(|e| e.to_string())
}

/// 从备份提取 Machine ID
#[tauri::command]
#[specta::specta]
pub async fn extract_backup_ids(
    service: State<'_, IdentityService>,
    backup_path: String,
) -> Result<MachineIds, String> {
    service.extract_ids_from_backup(&backup_path)
        .map_err(|e| e.to_string())
}

/// 删除备份文件
#[tauri::command]
#[specta::specta]
pub async fn delete_backup(backup_path: String) -> Result<serde_json::Value, String> {
    match std::fs::remove_file(&backup_path) {
        Ok(_) => {
            log_info!("成功删除备份文件: {}", backup_path);
            Ok(serde_json::json!({"success": true, "message": "备份文件删除成功"}))
        }
        Err(e) => {
            log_error!("删除备份文件失败: {}", e);
            Ok(serde_json::json!({"success": false, "message": format!("删除失败: {}", e)}))
        }
    }
}

/// 恢复 Machine ID
#[tauri::command]
#[specta::specta]
pub async fn restore_machine_ids(
    service: State<'_, IdentityService>,
    backup_path: String,
) -> Result<RestoreResult, String> {
    service.restore_from_backup(&backup_path)
        .map_err(|e| e.to_string())
}

/// 重置 Machine ID
#[tauri::command]
#[specta::specta]
pub async fn reset_machine_ids(
    service: State<'_, IdentityService>,
) -> Result<ResetResult, String> {
    service.reset()
        .map_err(|e| e.to_string())
}

/// 获取 Cursor 路径
#[tauri::command]
#[specta::specta]
pub async fn get_cursor_paths(
    service: State<'_, IdentityService>,
) -> Result<(String, String), String> {
    Ok(service.get_cursor_paths())
}

/// 检查 Cursor 安装
#[tauri::command]
#[specta::specta]
pub async fn check_cursor_installation(
    service: State<'_, IdentityService>,
) -> Result<bool, String> {
    Ok(service.check_installation())
}

/// 设置自定义 Cursor 路径
#[tauri::command]
#[specta::specta]
pub async fn set_custom_cursor_path(
    service: State<'_, IdentityService>,
    path: String,
) -> Result<String, String> {
    service.set_custom_path(&path)
        .map_err(|e| e.to_string())
}

/// 获取自定义 Cursor 路径
#[tauri::command]
#[specta::specta]
pub async fn get_custom_cursor_path(
    service: State<'_, IdentityService>,
) -> Result<Option<String>, String> {
    Ok(service.get_custom_path())
}

/// 清除自定义 Cursor 路径
#[tauri::command]
#[specta::specta]
pub async fn clear_custom_cursor_path(
    service: State<'_, IdentityService>,
) -> Result<String, String> {
    service.clear_custom_path()
        .map_err(|e| e.to_string())
}

/// 完全重置（重置 ID + 修改 main.js + workbench.js）
///
/// 除了重置 Machine ID，还修补 main.js 中的 getMachineId 函数
/// 使其不再返回硬编码的原始 ID。
#[tauri::command]
#[specta::specta]
pub async fn complete_cursor_reset(
    service: State<'_, IdentityService>,
) -> Result<ResetResult, String> {
    let mut result = service.reset().map_err(|e| e.to_string())?;

    // 修改 main.js
    if let Some(main_js) = &service.cursor().paths.main_js {
        if main_js.exists() {
            match modify_main_js(main_js) {
                Ok(_) => result.details.push("main.js 已修补".to_string()),
                Err(e) => result.details.push(format!("main.js 修补失败: {}", e)),
            }
        }
    }

    Ok(result)
}

/// 修改 main.js，移除 getMachineId/getMacMachineId 的硬编码返回
fn modify_main_js(path: &std::path::Path) -> Result<(), String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup = format!("{}.backup.{}", path.display(), timestamp);
    std::fs::copy(path, &backup).map_err(|e| e.to_string())?;

    let patterns = vec![
        (
            regex::Regex::new(r"async getMachineId\(\)\{return [^??]+\?\?([^}]+)\}").unwrap(),
            "async getMachineId(){return $1}",
        ),
        (
            regex::Regex::new(r"async getMacMachineId\(\)\{return [^??]+\?\?([^}]+)\}").unwrap(),
            "async getMacMachineId(){return $1}",
        ),
    ];

    let mut modified = content.clone();
    for (re, replacement) in &patterns {
        modified = re.replace_all(&modified, *replacement).to_string();
    }

    std::fs::write(path, &modified).map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取 machineId 文件内容
#[tauri::command]
#[specta::specta]
pub async fn get_machine_id_file_content() -> Result<Option<String>, String> {
    let machine_id_path = {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
            std::path::PathBuf::from(appdata).join("Cursor").join("machineId")
        }
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir().ok_or("无法获取 home 目录".to_string())?
                .join("Library/Application Support/Cursor/machineId")
        }
        #[cfg(target_os = "linux")]
        {
            dirs::home_dir().ok_or("无法获取 home 目录".to_string())?
                .join(".config/Cursor/machineId")
        }
    };

    if machine_id_path.exists() {
        let content = std::fs::read_to_string(&machine_id_path).map_err(|e| e.to_string())?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

/// 获取备份目录信息
#[tauri::command]
#[specta::specta]
pub async fn get_backup_directory_info(
    service: State<'_, IdentityService>,
) -> Result<(String, Vec<String>), String> {
    let (storage_path, _) = service.get_cursor_paths();
    let dir = std::path::Path::new(&storage_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let files = if std::path::Path::new(&dir).exists() {
        std::fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect()
    } else {
        Vec::new()
    };

    Ok((dir, files))
}

/// 获取自动更新状态
#[tauri::command]
#[specta::specta]
pub async fn get_auto_update_status() -> Result<serde_json::Value, String> {
    let updater_path = get_cursor_updater_path()?;
    let exists = updater_path.exists();
    Ok(serde_json::json!({
        "disabled": !exists,
        "path": updater_path.to_string_lossy(),
        "exists": exists
    }))
}

/// 禁用自动更新
#[tauri::command]
#[specta::specta]
pub async fn disable_auto_update() -> Result<serde_json::Value, String> {
    let updater_path = get_cursor_updater_path()?;
    if updater_path.exists() {
        let disabled_path = updater_path.with_extension("disabled");
        std::fs::rename(&updater_path, &disabled_path).map_err(|e| e.to_string())?;
        log_info!("已禁用自动更新");
        Ok(serde_json::json!({"success": true, "message": "已禁用自动更新"}))
    } else {
        Ok(serde_json::json!({"success": true, "message": "更新器不存在，无需禁用"}))
    }
}

/// 启用自动更新
#[tauri::command]
#[specta::specta]
pub async fn enable_auto_update() -> Result<serde_json::Value, String> {
    let updater_path = get_cursor_updater_path()?;
    let disabled_path = updater_path.with_extension("disabled");
    if disabled_path.exists() {
        std::fs::rename(&disabled_path, &updater_path).map_err(|e| e.to_string())?;
        log_info!("已启用自动更新");
        Ok(serde_json::json!({"success": true, "message": "已启用自动更新"}))
    } else {
        Ok(serde_json::json!({"success": true, "message": "更新器未被禁用"}))
    }
}

/// 调试 Cursor 路径
#[tauri::command]
#[specta::specta]
pub async fn debug_cursor_paths(
    service: State<'_, IdentityService>,
) -> Result<Vec<String>, String> {
    let (storage, sqlite) = service.get_cursor_paths();
    Ok(vec![
        format!("storage.json: {}", storage),
        format!("state.vscdb: {}", sqlite),
        format!("storage.json 存在: {}", std::path::Path::new(&storage).exists()),
        format!("state.vscdb 存在: {}", std::path::Path::new(&sqlite).exists()),
    ])
}

/// 调试 Windows Cursor 路径
#[tauri::command]
#[specta::specta]
pub async fn debug_windows_cursor_paths() -> Result<Vec<String>, String> {
    let mut info = Vec::new();
    #[cfg(target_os = "windows")]
    {
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "未设置".to_string());
        info.push(format!("LOCALAPPDATA: {}", localappdata));
    }
    #[cfg(not(target_os = "windows"))]
    {
        info.push("此功能仅在 Windows 上可用".to_string());
    }
    Ok(info)
}

/// 获取 cursor-updater 路径
fn get_cursor_updater_path() -> Result<std::path::PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let localappdata = std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?;
        Ok(std::path::PathBuf::from(localappdata).join("cursor-updater").join("cursor-updater.exe"))
    }
    #[cfg(target_os = "macos")]
    {
        Ok(std::path::PathBuf::from("/Applications/Cursor.app/Contents/Frameworks/Cursor Helper (Renderer).app/Contents/MacOS/Cursor Helper (Renderer)"))
    }
    #[cfg(target_os = "linux")]
    {
        Ok(std::path::PathBuf::from("/usr/bin/cursor-updater"))
    }
}

/// 启动 Cursor 应用
#[tauri::command]
#[specta::specta]
pub async fn launch_cursor() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "windows")]
    {
        let localappdata = std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?;
        let cursor_exe = std::path::PathBuf::from(&localappdata)
            .join("Programs").join("Cursor").join("Cursor.exe");
        if cursor_exe.exists() {
            std::process::Command::new(&cursor_exe)
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok(serde_json::json!({"success": true, "message": "Cursor 已启动"}));
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-a").arg("Cursor")
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(serde_json::json!({"success": true, "message": "Cursor 已启动"}));
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("cursor")
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(serde_json::json!({"success": true, "message": "Cursor 已启动"}));
    }

    #[allow(unreachable_code)]
    Ok(serde_json::json!({"success": false, "message": "未找到 Cursor"}))
}
