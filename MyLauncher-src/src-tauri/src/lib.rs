// ════════════════════════════════════════════
// MyLauncher - Rust 后端实现
// 绿色便携程序管理器
//
// 本文件只做三件事：声明模块、持有应用入口 run()、装配系统托盘。
// 具体业务按职责分散到各模块，见下方 mod 声明。
// ════════════════════════════════════════════

#![allow(unused_imports)]

mod models;
mod defaults;
mod paths;
mod storage;
mod launch;
mod icons;
mod icons_refresh;
mod icons_system;
mod steam;
mod import;
mod favicon;
mod bookmarks;
mod assets;
mod autorun;
mod lang;
mod window;

use tauri::{command, Emitter};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[cfg(windows)]
use windows::{
    core::{GUID, PCWSTR, Interface, HSTRING},
    Win32::{
        Foundation::{BOOL, HWND, SIZE},
        Graphics::Gdi::*,
        Storage::FileSystem::WIN32_FIND_DATAW,
        System::Com::{CoInitializeEx, CoUninitialize, CoCreateInstance, COINIT_APARTMENTTHREADED, CLSCTX_INPROC_SERVER, IPersistFile, STGM},
        UI::{
            Shell::{ShellExecuteW, IShellItemImageFactory, IShellLinkW, SHCreateItemFromParsingName, SIIGBF_ICONONLY, SHFILEOPSTRUCTW, SHFileOperationW},
            WindowsAndMessaging::*,
        },
    },
};

use crate::models::*;
use crate::defaults::*;
use crate::paths::*;
use crate::storage::*;
use crate::launch::*;
use crate::icons::*;
use crate::icons_refresh::*;
use crate::icons_system::*;
use crate::steam::*;
use crate::import::*;
use crate::favicon::*;
use crate::bookmarks::*;
use crate::assets::*;
use crate::autorun::*;
use crate::lang::*;
use crate::window::*;

// ════════════════════════════════════════════
// 应用入口
// ════════════════════════════════════════════

/// 托盘菜单项 ID
const TRAY_MENU_SHOW: &str = "show";
const TRAY_MENU_QUIT: &str = "quit";

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::{
                menu::{Menu, MenuItem},
                tray::{TrayIconBuilder, TrayIconEvent},
                Manager,
            };

            // ─── 系统托盘：图标 + 右键菜单 ───
            let show_item = MenuItem::with_id(app, TRAY_MENU_SHOW, "恢复窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, TRAY_MENU_QUIT, "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        // 恢复窗口：显示 + 取消最小化 + 抢占前台
                        TRAY_MENU_SHOW => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.unminimize();
                                let _ = win.set_focus();
                            }
                        }
                        // 退出：清理托盘图标后真正退出
                        TRAY_MENU_QUIT => {
                            if let Some(tray) = app.tray_by_id("main-tray") {
                                let _ = tray.set_visible(false);
                            }
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|_tray, event| {
                    // 左键单击托盘图标也恢复窗口（最小化状态下常用）
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } = event {
                        if let Some(win) = _tray.app_handle().get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.unminimize();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ─── 启动显示策略 ───
            // 窗口在 tauri.conf.json 中默认不可见；手动双击启动（无 --hidden 参数）显示主窗口，
            // 开机自启静默模式（带 --hidden 参数）保持隐藏只留托盘
            if let Some(win) = app.get_webview_window("main") {
                let silent_start = std::env::args().any(|a| a == AUTORUN_HIDDEN_ARG);
                if !silent_start {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }

            // ─── 自启路径校正：程序移动位置后用当前路径重写注册表（保留静默标记）───
            fix_autorun_registry();

            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::Manager;
            // 主窗口关闭请求 → 按设置分流
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    // 读取关闭行为设置（每次关闭时读取，开销可接受）
                    let close_action = get_data_dir()
                        .ok()
                        .and_then(|dir| read_json_file::<AppConfig>(&dir.join("config.json")).ok())
                        .map(|cfg| cfg.close_action)
                        .unwrap_or_else(default_close_action);

                    if close_action == "close" {
                        // 直接退出：清理托盘图标后关闭窗口（触发默认退出流程）
                        if let Some(tray) = window.app_handle().tray_by_id("main-tray") {
                            let _ = tray.set_visible(false);
                        }
                        // 不调用 prevent_close，让窗口正常关闭退出
                    } else {
                        // 默认：隐藏到托盘
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            init_data_dir,
            load_all_data,
            save_entries,
            save_categories,
            save_environments,
            save_config,
            get_exe_dir,
            get_now,
            resolve_path,
            check_paths_batch,
            launch_program,
            open_external_url,
            get_exe_info,
            get_file_info,
            resolve_lnk,
            scan_directory,
            scan_steam_app,
            import_scan_items,
            convert_path_mode,
            copy_file_to_assets,
            fetch_favicon,
            parse_bookmarks,
            list_appx_apps,
            extract_appx_icon,
            extract_system_icon,
            open_in_explorer,
            scan_orphan_assets,
            clean_orphan_assets,
            cleanup_deleted_assets,
            refresh_entry_icon,
            batch_refresh_icons,
            set_autorun,
            get_autorun,
            init_lang_dir,
            list_languages,
            read_language,
            open_lang_dir,
            set_window_title
        ])
        .run(tauri::generate_context!())
        .expect("MyLauncher 启动失败");
}
