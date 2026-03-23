/// 账号管理命令入口
///
/// 使用新架构 services::AccountService 处理业务逻辑。
use crate::domain::account::*;
use crate::services::account_service::AccountService;
use crate::{log_info, log_error};
use tauri::State;

/// 获取当前账号
#[tauri::command]
#[specta::specta]
pub async fn get_current_account(
    service: State<'_, AccountService>,
) -> Result<Option<AccountInfo>, String> {
    service.get_current()
        .map_err(|e| e.to_string())
}

/// 获取账号列表
#[tauri::command]
#[specta::specta]
pub async fn get_account_list(
    service: State<'_, AccountService>,
) -> Result<AccountListResult, String> {
    service.list_all()
        .map_err(|e| e.to_string())
}

/// 切换账号
#[tauri::command]
#[specta::specta]
pub async fn switch_account(
    service: State<'_, AccountService>,
    email: String,
) -> Result<SwitchAccountResult, String> {
    service.switch(&email)
        .map_err(|e| e.to_string())
}

/// 删除账号
#[tauri::command]
#[specta::specta]
pub async fn remove_account(
    service: State<'_, AccountService>,
    email: String,
) -> Result<serde_json::Value, String> {
    service.remove(&email)
        .map(|_| serde_json::json!({"success": true, "message": format!("已删除 {}", email)}))
        .map_err(|e| e.to_string())
}

/// 登出当前账号
#[tauri::command]
#[specta::specta]
pub async fn logout_current_account(
    service: State<'_, AccountService>,
) -> Result<LogoutResult, String> {
    service.logout()
        .map_err(|e| e.to_string())
}

/// 导出账号
#[tauri::command]
#[specta::specta]
pub async fn export_accounts(
    service: State<'_, AccountService>,
    export_path: String,
    selected_emails: Option<Vec<String>>,
) -> Result<serde_json::Value, String> {
    service.export(&export_path, selected_emails)
        .map_err(|e| e.to_string())
}

/// 导入账号
#[tauri::command]
#[specta::specta]
pub async fn import_accounts(
    service: State<'_, AccountService>,
    import_file_path: String,
) -> Result<serde_json::Value, String> {
    service.import(&import_file_path)
        .map_err(|e| e.to_string())
}

/// 检查管理员权限
#[tauri::command]
#[specta::specta]
pub async fn check_admin_privileges() -> Result<bool, String> {
    let platform = crate::infra::platform::create();
    Ok(platform.is_admin())
}

/// 添加账号
#[tauri::command]
#[specta::specta]
pub async fn add_account(
    service: State<'_, AccountService>,
    email: String,
    token: String,
    refresh_token: Option<String>,
    workos_cursor_session_token: Option<String>,
    username: Option<String>,
    tags: Option<Vec<String>>,
    machine_ids_json: Option<String>,
) -> Result<serde_json::Value, String> {
    let machine_ids = machine_ids_json
        .and_then(|json| serde_json::from_str(&json).ok())
        .or_else(|| service.cursor().read_full_machine_ids().ok());

    let account = AccountInfo {
        email: email.clone(),
        token,
        refresh_token,
        workos_cursor_session_token,
        is_current: false,
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        username,
        tags: tags.unwrap_or_default(),
        machine_ids,
        subscription_type: None,
        subscription_status: None,
        trial_days_remaining: None,
        name: None,
        sub: None,
        picture: None,
        user_id: None,
    };

    service.add(account)
        .map(|_| serde_json::json!({"success": true, "message": format!("账号 {} 添加成功", email)}))
        .map_err(|e| e.to_string())
}

/// 编辑账号
#[tauri::command]
#[specta::specta]
pub async fn edit_account(
    service: State<'_, AccountService>,
    email: String,
    new_email: Option<String>,
    new_token: Option<String>,
    new_refresh_token: Option<String>,
    new_workos_cursor_session_token: Option<String>,
    new_username: Option<String>,
    new_tags: Option<Vec<String>>,
    new_machine_ids: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let mut accounts = service.store().load_all().map_err(|e| e.to_string())?;

    if let Some(acc) = accounts.iter_mut().find(|a| a.email == email) {
        if let Some(ne) = new_email { acc.email = ne; }
        if let Some(nt) = new_token { acc.token = nt; }
        if let Some(nr) = new_refresh_token { acc.refresh_token = Some(nr); }
        if let Some(nw) = new_workos_cursor_session_token { acc.workos_cursor_session_token = Some(nw); }
        if let Some(nu) = new_username { acc.username = Some(nu); }
        if let Some(nt) = new_tags { acc.tags = nt; }
        if let Some(nm) = new_machine_ids {
            acc.machine_ids = serde_json::from_value(nm).ok();
        }
        service.store().save_all(&accounts).map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"success": true, "message": "账号编辑成功"}))
    } else {
        Ok(serde_json::json!({"success": false, "message": format!("账号 {} 不存在", email)}))
    }
}

/// 带选项切换账号
///
/// 根据选项处理机器码：
/// - use_bound_machine_id=true：恢复账号绑定的机器码到所有存储位置
/// - reset_machine_id=true：生成新机器码并写入所有存储位置，同时更新账号绑定
/// - 都为 false：保持当前机器码不变
#[tauri::command]
#[specta::specta]
pub async fn switch_account_with_options(
    service: State<'_, AccountService>,
    email: String,
    reset_machine_id: bool,
    use_bound_machine_id: bool,
) -> Result<SwitchAccountResult, String> {
    let cursor = service.cursor();

    if use_bound_machine_id {
        let accounts = service.store().load_all().map_err(|e| e.to_string())?;
        if let Some(account) = accounts.iter().find(|a| a.email == email) {
            if let Some(ref ids) = account.machine_ids {
                let _ = cursor.storage().write_machine_ids(ids);
                let _ = cursor.sqlite().update_service_machine_id(&ids.service_machine_id);
                let platform = crate::infra::platform::create();
                let _ = platform.update_system_ids(ids);
            }
        }
    } else if reset_machine_id {
        let new_ids = crate::domain::identity::MachineIds::generate();
        let _ = cursor.storage().write_machine_ids(&new_ids);
        let _ = cursor.sqlite().update_service_machine_id(&new_ids.service_machine_id);
        let platform = crate::infra::platform::create();
        let _ = platform.update_system_ids(&new_ids);

        // 将新生成的机器码保存到该账号的绑定数据
        let mut accounts = service.store().load_all().map_err(|e| e.to_string())?;
        if let Some(acc) = accounts.iter_mut().find(|a| a.email == email) {
            let mut bound_ids = new_ids;
            let (guid, sqm) = crate::infra::platform::read_registry_ids();
            bound_ids.machine_guid = guid;
            bound_ids.sqm_client_id = sqm;
            acc.machine_ids = Some(bound_ids);
            let _ = service.store().save_all(&accounts);
        }
    }

    service.switch(&email).map_err(|e| e.to_string())
}

/// 使用 token 直接切换（无缝切号场景）
#[tauri::command]
#[specta::specta]
pub async fn switch_account_with_token(
    service: State<'_, AccountService>,
    email: String,
    token: String,
    auth_type: Option<String>,
) -> Result<SwitchAccountResult, String> {
    let cursor = service.cursor();
    let clean_token = crate::infra::api::checksum::TokenParser::extract_token_part(&token);

    let _ = cursor.storage().write_auth(&email, &clean_token);
    let _ = cursor.sqlite().inject_email(&email);
    let _ = cursor.sqlite().inject_token(&clean_token);

    Ok(SwitchAccountResult {
        success: true,
        message: format!("已通过 Token 切换至 {}", email),
        details: vec!["Token 直接注入完成".to_string()],
    })
}

/// 刷新单个账号信息（通过 Token 查询授权 + 订阅状态）
///
/// 返回完整的 AuthCheckResult，与前端 configService.refreshSingleAccountInfo 对接。
#[tauri::command]
#[specta::specta]
pub async fn refresh_single_account_info(token: String) -> Result<serde_json::Value, String> {
    match crate::commands::system::check_user_authorization(token).await {
        Ok(result) => Ok(serde_json::json!(result)),
        Err(e) => Err(format!("获取账户信息失败: {}", e)),
    }
}

/// 批量刷新账号信息
#[tauri::command]
#[specta::specta]
pub async fn refresh_all_accounts_info(tokens: Vec<String>) -> Result<serde_json::Value, String> {
    let mut results = Vec::new();

    for token in &tokens {
        match crate::commands::system::check_user_authorization(token.clone()).await {
            Ok(result) => {
                results.push(serde_json::json!({
                    "token": token,
                    "success": result.success,
                    "user_info": result.user_info
                }));
            }
            Err(e) => {
                results.push(serde_json::json!({
                    "token": token,
                    "success": false,
                    "error": e.to_string()
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "success": true,
        "results": results,
        "total": tokens.len()
    }))
}

/// 获取已保存账号列表（原始 JSON）
#[tauri::command]
#[specta::specta]
pub async fn get_saved_accounts() -> Result<Vec<serde_json::Value>, String> {
    let path = crate::get_data_dir()?.join("account_cache.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let accounts: Vec<serde_json::Value> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(accounts)
}

/// 列出 Windows 用户
#[tauri::command]
#[specta::specta]
pub async fn list_windows_users() -> Result<serde_json::Value, String> {
    #[cfg(target_os = "windows")]
    {
        let users_dir = std::path::PathBuf::from("C:\\Users");
        if !users_dir.exists() {
            return Ok(serde_json::json!({"success": false, "message": "C:\\Users 不存在"}));
        }

        let current_user = std::env::var("USERNAME").unwrap_or_default();
        let mut users = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&users_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let skip = ["Public", "Default", "Default User", "All Users", "desktop.ini"];
                if skip.contains(&name.as_str()) { continue; }

                if entry.path().is_dir() {
                    let cursor_dir = entry.path().join("AppData").join("Roaming").join("Cursor");
                    users.push(serde_json::json!({
                        "username": name,
                        "is_current": name == current_user,
                        "has_cursor": cursor_dir.exists(),
                        "path": entry.path().to_string_lossy(),
                    }));
                }
            }
        }

        Ok(serde_json::json!({"success": true, "users": users}))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(serde_json::json!({"success": false, "message": "仅 Windows 支持"}))
    }
}

/// 同步账号到其他 Windows 用户
#[tauri::command]
#[specta::specta]
pub async fn sync_account_to_user(target_username: String) -> Result<serde_json::Value, String> {
    #[cfg(target_os = "windows")]
    {
        let source_appdata = std::env::var("APPDATA").map_err(|e| e.to_string())?;
        let source_storage = std::path::PathBuf::from(&source_appdata)
            .join("Cursor").join("User").join("globalStorage").join("storage.json");

        let target_storage = std::path::PathBuf::from("C:\\Users")
            .join(&target_username)
            .join("AppData").join("Roaming")
            .join("Cursor").join("User").join("globalStorage").join("storage.json");

        if !source_storage.exists() {
            return Ok(serde_json::json!({"success": false, "message": "源 storage.json 不存在"}));
        }

        if let Some(parent) = target_storage.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        std::fs::copy(&source_storage, &target_storage).map_err(|e| e.to_string())?;

        let source_account = crate::get_data_dir()?.join("account_cache.json");
        if source_account.exists() {
            let target_account = std::path::PathBuf::from("C:\\Users")
                .join(&target_username)
                .join("AppData").join("Roaming")
                .join("Cursor").join("User").join("globalStorage")
                .join("account_cache.json");
            let _ = std::fs::copy(&source_account, &target_account);
        }

        crate::log_info!("已同步到用户: {}", target_username);
        Ok(serde_json::json!({"success": true, "message": format!("已同步到 {}", target_username)}))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(serde_json::json!({"success": false, "message": "仅 Windows 支持"}))
    }
}
