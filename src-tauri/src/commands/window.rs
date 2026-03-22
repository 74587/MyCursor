/// 窗口管理命令
///
/// 涉及 Tauri 窗口创建、Cookie 注入、JavaScript 执行等。
/// 这些命令需要 `tauri::AppHandle` 参数。
use crate::{log_info, log_error};
use tauri::{Emitter, Manager};

/// 打开取消订阅页面
#[tauri::command]
#[specta::specta]
pub async fn open_cancel_subscription_page(
    app: tauri::AppHandle,
    session_token: String,
    stripe_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let url = stripe_url.unwrap_or_else(|| "https://www.cursor.com/settings".to_string());
    let cookie = crate::infra::api::CursorApiClient::build_workos_cookie(&session_token);

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "cancel_subscription",
        tauri::WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?),
    )
    .title("Cursor 订阅管理")
    .inner_size(1200.0, 800.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success": true, "message": "已打开订阅管理页面"}))
}

/// 显示取消订阅窗口
#[tauri::command]
#[specta::specta]
pub async fn show_cancel_subscription_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("cancel_subscription") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

/// 取消订阅失败
#[tauri::command]
#[specta::specta]
pub async fn cancel_subscription_failed(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("cancel_subscription") {
        let _ = window.close();
    }
    Ok(())
}

/// 打开绑卡信息页面
#[tauri::command]
#[specta::specta]
pub async fn open_bind_card_info(
    app: tauri::AppHandle,
    session_token: String,
    stripe_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let url = stripe_url.unwrap_or_else(|| "https://www.cursor.com/settings".to_string());

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "bind_card",
        tauri::WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?),
    )
    .title("绑卡信息")
    .inner_size(1200.0, 800.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success": true, "message": "已打开绑卡页面"}))
}

/// 删除 Cursor 账户
#[tauri::command]
#[specta::specta]
pub async fn delete_cursor_account(
    session_token: String,
    email: String,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let cookie = crate::infra::api::CursorApiClient::build_workos_cookie(&session_token);

    let resp = client
        .post("https://www.cursor.com/api/dashboard/delete-account")
        .header("Cookie", &cookie)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({"email": email}))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status().as_u16();
    if status == 200 {
        log_info!("Cursor 账户 {} 已注销", email);
        Ok(serde_json::json!({"success": true, "message": format!("账户 {} 已注销", email)}))
    } else {
        let text = resp.text().await.unwrap_or_default();
        Ok(serde_json::json!({"success": false, "status": status, "message": text}))
    }
}

/// 生成 PKCE 登录参数（UUID + verifier + challenge）
///
/// 后端统一生成，前端只需使用返回值。
#[tauri::command]
#[specta::specta]
pub async fn generate_pkce_params() -> Result<serde_json::Value, String> {
    use sha2::{Digest, Sha256};
    use base64::{Engine as _, engine::general_purpose};

    let uuid = uuid::Uuid::new_v4().to_string();
    let verifier = uuid::Uuid::new_v4().to_string();

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

    let login_url = format!(
        "https://cursor.com/cn/loginDeepControl?challenge={}&uuid={}&mode=login",
        challenge, uuid
    );

    Ok(serde_json::json!({
        "uuid": uuid,
        "verifier": verifier,
        "challenge": challenge,
        "login_url": login_url
    }))
}

/// 触发 PKCE 授权登录
///
/// 使用后端生成的 uuid 和 challenge，加上 WorkOS Session Token，向 Cursor 认证 API 发送登录请求。
#[tauri::command]
#[specta::specta]
pub async fn trigger_authorization_login(
    uuid: String,
    challenge: String,
    workos_cursor_session_token: String,
) -> Result<serde_json::Value, String> {
    use reqwest::header::{HeaderMap, HeaderValue};

    log_info!("开始调用 Cursor 授权登录 API...");

    let mut headers = HeaderMap::new();
    let cookie_value = format!("WorkosCursorSessionToken={}", workos_cursor_session_token);
    headers.insert(
        "Cookie",
        HeaderValue::from_str(&cookie_value).map_err(|e| format!("Invalid cookie: {}", e))?,
    );

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "challenge": challenge,
        "uuid": uuid,
    });

    match client
        .post("https://cursor.com/api/auth/loginDeepCallbackControl")
        .headers(headers)
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let headers_map: std::collections::HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            match response.text().await {
                Ok(body) => Ok(serde_json::json!({
                    "success": status.is_success(),
                    "status": status.as_u16(),
                    "message": if status.is_success() {
                        format!("授权登录请求成功！状态码: {}", status)
                    } else {
                        format!("授权登录失败！状态码: {}, 响应: {}", status, body)
                    },
                    "response_body": body,
                    "response_headers": headers_map
                })),
                Err(e) => Ok(serde_json::json!({
                    "success": false,
                    "status": status.as_u16(),
                    "message": format!("读取响应失败: {}", e),
                    "response_headers": headers_map
                })),
            }
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "message": format!("网络请求失败: {}", e)
        })),
    }
}

/// PKCE 登录轮询
///
/// 使用 UUID 和 verifier 向 Cursor 认证服务器轮询获取 token。
#[tauri::command]
#[specta::specta]
pub async fn trigger_authorization_login_poll(
    uuid: String,
    verifier: String,
) -> Result<serde_json::Value, String> {
    use reqwest::header::{HeaderMap, HeaderValue};

    log_info!("开始调用 Cursor 授权登录 Poll API...");

    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_static("*/*"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("Origin", HeaderValue::from_static("https://cursor.com"));
    headers.insert("User-Agent", HeaderValue::from_static(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36"
    ));

    let client = reqwest::Client::new();

    match client
        .get(&format!(
            "https://api2.cursor.sh/auth/poll?uuid={}&verifier={}",
            uuid, verifier
        ))
        .headers(headers)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let headers_map: std::collections::HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            match response.text().await {
                Ok(body) => Ok(serde_json::json!({
                    "success": status.is_success(),
                    "status": status.as_u16(),
                    "message": if status.is_success() {
                        format!("授权登录Poll成功！状态码: {}", status)
                    } else {
                        format!("授权登录Poll失败！状态码: {}, 响应: {}", status, body)
                    },
                    "response_body": body,
                    "response_headers": headers_map
                })),
                Err(e) => Ok(serde_json::json!({
                    "success": false,
                    "status": status.as_u16(),
                    "message": format!("读取响应失败: {}", e),
                    "response_headers": headers_map
                })),
            }
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "message": format!("网络请求失败: {}", e)
        })),
    }
}

/// 打开登录获取 Session Token
#[tauri::command]
#[specta::specta]
pub async fn open_login_for_session_token(
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "login_session",
        tauri::WebviewUrl::External("https://authenticator.cursor.sh/".parse().unwrap()),
    )
    .title("Cursor 登录")
    .inner_size(1200.0, 800.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success": true, "message": "已打开登录窗口"}))
}

/// 自动登录并获取 Cookie
#[tauri::command]
#[specta::specta]
pub async fn auto_login_and_get_cookie(
    app: tauri::AppHandle,
    session_token: String,
    target_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let url = target_url.unwrap_or_else(|| "https://www.cursor.com/settings".to_string());

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "auto_login",
        tauri::WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?),
    )
    .title("Cursor")
    .inner_size(1200.0, 800.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success": true, "message": "已打开自动登录窗口"}))
}

/// 验证码登录
///
/// 打开验证码登录窗口。
#[tauri::command]
#[specta::specta]
pub async fn verification_code_login(
    app: tauri::AppHandle,
    email: String,
) -> Result<serde_json::Value, String> {
    let url = format!("https://authenticator.cursor.sh/sign-in?email={}", email);

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "verification_login",
        tauri::WebviewUrl::External(url.parse().map_err(|e: url::ParseError| e.to_string())?),
    )
    .title("验证码登录")
    .inner_size(800.0, 700.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success": true, "message": "已打开验证码登录窗口"}))
}

/// 检查验证码登录 Cookie
#[tauri::command]
#[specta::specta]
pub async fn check_verification_login_cookies(
    app: tauri::AppHandle,
) -> Result<(), String> {
    Ok(())
}

/// 检查登录 Cookie
#[tauri::command]
#[specta::specta]
pub async fn check_login_cookies(
    app: tauri::AppHandle,
) -> Result<(), String> {
    Ok(())
}

/// 自动登录成功
#[tauri::command]
#[specta::specta]
pub async fn auto_login_success(
    app: tauri::AppHandle,
    session_token: String,
    email: String,
) -> Result<serde_json::Value, String> {
    log_info!("自动登录成功: {}", email);
    Ok(serde_json::json!({"success": true, "email": email}))
}

/// 自动登录失败
#[tauri::command]
#[specta::specta]
pub async fn auto_login_failed(
    app: tauri::AppHandle,
    error: String,
) -> Result<(), String> {
    log_error!("自动登录失败: {}", error);
    Ok(())
}

/// 显示自动登录窗口
#[tauri::command]
#[specta::specta]
pub async fn show_auto_login_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("auto_login") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

/// 打开 Cursor Dashboard
#[tauri::command]
#[specta::specta]
pub async fn open_cursor_dashboard(
    app: tauri::AppHandle,
    session_token: String,
) -> Result<serde_json::Value, String> {
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "cursor_dashboard",
        tauri::WebviewUrl::External("https://www.cursor.com/settings".parse().unwrap()),
    )
    .title("Cursor Dashboard")
    .inner_size(1200.0, 800.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({"success": true, "message": "已打开 Dashboard"}))
}
