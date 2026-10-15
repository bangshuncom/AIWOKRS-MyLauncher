// ════════════════════════════════════════════
// 系统窗口标题同步
//
// 窗口标题栏不再硬编码产品口号（如"xxx - 绿色便携程序管理器"），
// 而是完全跟随设置里用户可自定义的名称（config.logo_text）：
// 前端启动时与每次改名后调用 set_window_title 同步。
// ════════════════════════════════════════════

use tauri::{command, Manager};

/// 默认窗口标题（用户未自定义 logo_text 时使用）
const DEFAULT_WINDOW_TITLE: &str = "MyLauncher";

/// 同步系统窗口标题为用户自定义标题（logo_text）。
/// 传空或纯空白时恢复默认标题。
#[command]
pub fn set_window_title(app: tauri::AppHandle, title: String) -> Result<(), String> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let trimmed = title.trim();
    let final_title = if trimmed.is_empty() {
        DEFAULT_WINDOW_TITLE.to_string()
    } else {
        trimmed.to_string()
    };
    win.set_title(&final_title).map_err(|e| e.to_string())
}
