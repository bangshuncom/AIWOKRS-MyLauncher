#![allow(unused_imports)]

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

use crate::assets::*;
use crate::autorun::*;
use crate::bookmarks::*;
use crate::defaults::*;
use crate::favicon::*;
use crate::icons::*;
use crate::icons_refresh::*;
use crate::icons_system::*;
use crate::import::*;
use crate::launch::*;
use crate::models::*;
use crate::paths::*;
use crate::storage::*;


// ═══════════════════════════════════════════
// Steam 游戏扫描
// ═══════════════════════════════════════════

// ─── 文件夹扫描 ───

/// 判断指定目录是否为 Steam 安装目录（存在 steamapps 子目录）
pub(crate) fn is_steam_dir(path: &Path) -> bool {
    path.join("steamapps").is_dir()
}

/// 读取 steamapps 目录下的所有 appmanifest_*.acf，解析出 Steam 游戏列表。
/// 每个条目包含 appid/name/installdir（解析失败的行跳过）。
pub(crate) fn read_steam_manifests(steamapps_dir: &Path) -> Vec<(String, String, String)> {
    let mut games = Vec::new();
    let entries = match std::fs::read_dir(steamapps_dir) {
        Ok(e) => e,
        Err(_) => return games,
    };

    for item in entries.flatten() {
        let path = item.path();
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !fname.starts_with("appmanifest_") || !fname.ends_with(".acf") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut appid = String::new();
        let mut name = String::new();
        let mut installdir = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            if let Some(v) = extract_acf_value(line, "appid") { appid = v.to_string(); }
            else if let Some(v) = extract_acf_value(line, "name") { name = v.to_string(); }
            else if let Some(v) = extract_acf_value(line, "installdir") { installdir = v.to_string(); }
        }
        if !appid.is_empty() && !name.is_empty() && !installdir.is_empty() {
            games.push((appid, name, installdir));
        }
    }
    games
}

/// 从 acf 行中提取指定 key 的字符串值（支持 "key" "value" 形式）
pub(crate) fn extract_acf_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(&format!("\"{}\"", key))?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// 在游戏安装目录下递归查找候选 exe（跳过常见无意义文件，如 unins*.exe /
/// Redist 目录 / win 开头文件）。返回全部候选，第一个视为主程序。
pub(crate) fn find_game_exes(install_dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(install_dir)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() { continue; }
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if ext != "exe" { continue; }
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if fname.starts_with("unins") { continue; }
        if fname.starts_with("win") && fname.ends_with(".exe") { continue; }
        // 跳过 redist/vcredist 等常见非游戏主程序
        let rel = path.strip_prefix(install_dir).unwrap_or(path);
        let rel_lower = rel.to_string_lossy().to_lowercase();
        if rel_lower.contains("redist") || rel_lower.contains("vcredist") || rel_lower.contains("directx") {
            continue;
        }
        found.push(path.to_string_lossy().to_string());
    }
    found
}

/// 尝试将 Steam 本地缓存的 header 图（appcache/librarycache/{appid}_header.jpg）复制到
/// assets/covers/steam/{appid}.jpg；缓存缺失时尝试从 CDN 下载（2 秒超时）。
/// 全部失败返回空字符串（条目不设封面）。
pub(crate) fn get_steam_cover(steam_install_dir: &Path, appid: &str) -> String {
    // 0. 目标已存在直接复用（同一游戏多 exe 并发导入时避免并发写同一文件失败）
    if let Ok(assets_dir) = get_assets_dir() {
        let target = assets_dir.join("covers").join("steam").join(format!("{}.jpg", appid));
        if target.is_file() {
            return rel_to_exe_dir(&target);
        }
    }

    // 1. 优先复制 Steam 本地缓存（用户明确要求优先用本地缓存）
    let cache_dir = steam_install_dir.join("appcache").join("librarycache");
    let cache_file = cache_dir.join(format!("{}_header.jpg", appid));
    if cache_file.is_file() {
        if let Ok(rel) = copy_steam_cover_to_assets(&cache_file, appid) {
            return rel;
        }
    }

    // 2. 本地缓存缺失 → 从 CDN 下载（与 favicon 相同的 2 秒超时客户端）
    let url = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{}/header.jpg", appid);
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let bytes = match client.get(&url).send().and_then(|r| r.bytes()) {
        Ok(b) => b,
        Err(_) => return String::new(),
    };
    if bytes.is_empty() || !bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return String::new();
    }

    let assets_dir = match get_assets_dir() {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let steam_dir = assets_dir.join("covers").join("steam");
    if ensure_dir(&steam_dir).is_err() {
        return String::new();
    }
    let target = steam_dir.join(format!("{}.jpg", appid));
    if std::fs::write(&target, &bytes).is_ok() {
        rel_to_exe_dir(&target)
    } else {
        String::new()
    }
}

/// 将本地 header 图片复制到 assets/covers/steam/{appid}.jpg，返回相对路径
pub(crate) fn copy_steam_cover_to_assets(src: &Path, appid: &str) -> Result<String, String> {
    let assets_dir = get_assets_dir()?;
    let steam_dir = assets_dir.join("covers").join("steam");
    ensure_dir(&steam_dir)?;
    let target = steam_dir.join(format!("{}.jpg", appid));
    std::fs::copy(src, &target)
        .map_err(|e| format!("复制封面失败: {}", e))?;
    Ok(rel_to_exe_dir(&target))
}

/// 从注册表 HKCU\Software\Valve\Steam 读取 Steam 安装路径（SteamPath 值）。
/// 未安装 Steam 或读取失败返回 None。
#[cfg(windows)]
pub(crate) fn get_steam_install_dir_from_registry() -> Option<PathBuf> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{RegCloseKey, RegOpenKeyW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER};

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    unsafe {
        let sub_key = to_wide(r"Software\Valve\Steam");
        let value_name = to_wide("SteamPath");
        let mut key = HKEY::default();
        // 只读打开（不创建）：未安装 Steam 时返回失败
        if RegOpenKeyW(HKEY_CURRENT_USER, PCWSTR(sub_key.as_ptr()), &mut key).is_err() {
            return None;
        }
        let mut buf_len: u32 = 0;
        let query = RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            None,
            None,
            Some(&mut buf_len),
        );
        if query.is_err() || buf_len == 0 {
            let _ = RegCloseKey(key);
            return None;
        }
        let mut buf = vec![0u8; buf_len as usize];
        let result = RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            None,
            Some(buf.as_mut_ptr()),
            Some(&mut buf_len),
        );
        let _ = RegCloseKey(key);
        if result.is_err() {
            return None;
        }
        // 字节 → UTF-16 字符串（去掉结尾 NUL）
        let words: Vec<u16> = buf
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let s = String::from_utf16_lossy(&words);
        let s = s.trim_end_matches('\0');
        if s.is_empty() { None } else { Some(PathBuf::from(s)) }
    }
}


/// 解析主 Steam 目录下 steamapps/libraryfolders.vdf，返回所有库的 steamapps 目录列表
/// （含主库）。文件缺失或解析失败时仅返回主库。
pub(crate) fn read_steam_library_dirs(steam_dir: &Path) -> Vec<PathBuf> {
    let main_steamapps = steam_dir.join("steamapps");
    let mut libs = vec![main_steamapps.clone()];

    let vdf_path = main_steamapps.join("libraryfolders.vdf");
    let content = match std::fs::read_to_string(&vdf_path) {
        Ok(c) => c,
        Err(_) => return libs,
    };
    // 逐行提取 "path" 值（vdf KeyValues 格式，与 acf 相同的行级 key-value）
    // 路径中的 \\ 转义还原为 \
    for line in content.lines() {
        if let Some(p) = extract_acf_value(line.trim(), "path") {
            if p.is_empty() { continue; }
            let dir = PathBuf::from(p.replace("\\\\", "\\")).join("steamapps");
            // 去重：主库通常也出现在 vdf 中
            if !libs.contains(&dir) {
                libs.push(dir);
            }
        }
    }
    libs
}

/// 按 Steam 应用 ID 查找已安装游戏的扫描结果
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SteamAppScanResult {
    /// 游戏名称（appmanifest 的 name）
    pub name: String,
    /// 游戏 exe 绝对路径（安装目录下递归查找，可能为空）
    pub exe_path: String,
    /// 封面图相对路径（assets/covers/steam/{appid}.jpg，空表示未获取到）
    pub cover_path: String,
    /// 图标相对路径（从游戏 exe 提取，可能为空）
    pub icon_path: String,
}

/// 按 Steam 应用 ID 扫描单个已安装游戏。
/// 流程：注册表定位 Steam → 解析 libraryfolders.vdf 遍历所有库 →
/// 各库找 steamapps/appmanifest_{appid}.acf → 回填名称/exe/封面/图标。
/// 找不到 manifest 说明该 ID 游戏不存在或未安装。
#[command]
pub fn scan_steam_app(app_id: String) -> Result<SteamAppScanResult, String> {
    let app_id = app_id.trim().to_string();
    if app_id.is_empty() {
        return Err("请输入 Steam 游戏 ID".to_string());
    }

    let steam_dir = get_steam_install_dir_from_registry()
        .ok_or_else(|| "未检测到 Steam 客户端（注册表 HKCU\\Software\\Valve\\Steam）".to_string())?;
    if !steam_dir.is_dir() {
        return Err(format!("Steam 目录不存在: {}", steam_dir.display()));
    }

    // 遍历所有库找 appmanifest_{appid}.acf
    let manifest_file = format!("appmanifest_{}.acf", app_id);
    for lib_steamapps in read_steam_library_dirs(&steam_dir) {
        let manifest_path = lib_steamapps.join(&manifest_file);
        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut name = String::new();
        let mut installdir = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            if let Some(v) = extract_acf_value(line, "name") { name = v; }
            else if let Some(v) = extract_acf_value(line, "installdir") { installdir = v; }
        }
        if name.is_empty() || installdir.is_empty() {
            continue;
        }

        // 解析游戏信息（exe/封面/图标）——单游戏流程量小，保留即时提取
        let install_dir = lib_steamapps.join("common").join(&installdir);
        if !install_dir.is_dir() {
            return Err(format!("游戏目录不存在: {}", install_dir.display()));
        }
        let game_exe = find_game_exes(&install_dir).into_iter().next().unwrap_or_default();
        let cover_path = get_steam_cover(&steam_dir, &app_id);
        let icon_path = if !game_exe.is_empty() {
            extract_icon_to_png(&game_exe).unwrap_or_default()
        } else {
            String::new()
        };

        return Ok(SteamAppScanResult { name, exe_path: game_exe, cover_path, icon_path });
    }

    Err(format!("游戏 ID {} 不存在或未安装（所有 Steam 库中均未找到 appmanifest_{}.acf）", app_id, app_id))
}

/// 扫描目录（普通文件夹/Steam 安装目录）。
/// 扫描阶段不提取图标、不下载封面（保证速度），仅收集文件清单；
/// 图标/封面等重操作移到导入命令 import_scan_items 中执行。
/// 异步执行并推送 scan-progress 事件（前端进度弹窗实时展示）：
///   - phase="walking"  ：遍历文件阶段，payload 含 found（已发现的 exe 数）
///   - phase="icons"    ：解析阶段（Steam 游戏清单），payload 含 done/total
/// 事件名：scan-progress
#[command]
pub async fn scan_directory(app: tauri::AppHandle, dir_path: String, max_depth: Option<usize>) -> Result<ScanResult, String> {
    let path = Path::new(&dir_path);
    if !path.exists() || !path.is_dir() {
        return Err(format!("目录不存在: {}", dir_path));
    }

    // Steam 安装目录：直接解析 appmanifest 生成游戏条目（不递归扫 exe）
    if is_steam_dir(path) {
        let _ = app.emit("scan-progress", serde_json::json!({
            "phase": "walking",
            "found": 0,
            "done": 0,
            "total": 0,
        }));
        return scan_steam_games(&app, path);
    }

    let depth = max_depth.unwrap_or(3);
    let exe_dir = get_exe_directory()?;
    let mut results = Vec::new();

    // 阶段一：遍历目录收集 exe 文件（全程推送 found 计数）
    let mut walk_paths = Vec::new();
    for entry in walkdir::WalkDir::new(&dir_path)
        .max_depth(depth)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if ext != "exe" {
            continue;
        }
        let abs_path = path.to_string_lossy().to_string();
        walk_paths.push(abs_path);
        // 每发现一个 exe 推送一次（前端进度文字「正在扫描目录… 已发现 N 个程序」）
        let _ = app.emit("scan-progress", serde_json::json!({
            "phase": "walking",
            "found": walk_paths.len(),
            "done": 0,
            "total": 0,
        }));
    }

    let total = walk_paths.len();
    let mut done = 0usize;

        // 第二阶段：逐条解析名称与相对路径（图标提取移到导入阶段，扫描只收集清单，保证速度）
        for abs_path in &walk_paths {
            let path = Path::new(abs_path);
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未知")
                .to_string();

            let (suggested_mode, rel_path) = if let Some(rel) = make_relative_path(&exe_dir, path) {
                ("relative", rel)
            } else {
                ("absolute", String::new())
            };

            results.push(ExeInfo {
                name: name.clone(),
                icon_path: String::new(),
                suggested_path_mode: suggested_mode.to_string(),
                relative_path: rel_path,
                absolute_path: abs_path.clone(),
                file_exists: true,
                steam: None,
                multi_exe: false,
                is_primary: true,
            });

            done += 1;
            let _ = app.emit("scan-progress", serde_json::json!({
                "phase": "walking",
                "found": total,
                "done": done,
                "total": total,
                "current": name,
            }));
        }

    Ok(ScanResult { results, total })
}

/// 扫描 Steam 安装目录：解析 steamapps/appmanifest_*.acf，为每个游戏生成一个
/// ExeInfo（name 用游戏名，absolute_path 用找到的游戏主程序，steam 字段带 appid/
/// installdir/steam_dir）。扫描阶段不提取图标、不下载封面（保证扫描速度），
/// 多候选 exe 的游戏逐个展开为多条（multi_exe=true，第一个 is_primary=true）。
/// 逐游戏推送 scan-progress 事件（phase="icons"，含 done/total 与当前游戏名）。
pub(crate) fn scan_steam_games(app: &tauri::AppHandle, steam_dir: &Path) -> Result<ScanResult, String> {
    let steamapps = steam_dir.join("steamapps");
    let games = read_steam_manifests(&steamapps);
    let exe_dir = get_exe_directory()?;
    let mut results = Vec::new();

    let total = games.len();
    let mut done = 0usize;

    for (appid, name, installdir) in games {
        let install_dir = steamapps.join("common").join(&installdir);
        // 找出全部候选 exe：多个时逐个展开为结果条目（前端红色提示用户勾选），
        // 第一个为主程序（默认选中）；图标提取与封面下载移到导入阶段
        let all_exes = find_game_exes(&install_dir);

        for (exe_idx, game_exe) in all_exes.iter().enumerate() {
            let is_primary = exe_idx == 0;
            let multi_exe = all_exes.len() > 1;
            // 多 exe 时名称附 exe 名区分（“游戏名 [exe名]”），单 exe 保持游戏名
            let display_name = if multi_exe {
                let exe_stem = Path::new(game_exe)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                format!("{} [{}]", name, exe_stem)
            } else {
                name.clone()
            };

            let (suggested_mode, rel_path) = if !game_exe.is_empty() {
                if let Some(rel) = make_relative_path(&exe_dir, Path::new(game_exe)) {
                    ("relative", rel)
                } else {
                    ("absolute", String::new())
                }
            } else {
                ("absolute", String::new())
            };

            results.push(ExeInfo {
                name: display_name,
                icon_path: String::new(),
                suggested_path_mode: suggested_mode.to_string(),
                relative_path: rel_path,
                absolute_path: game_exe.clone(),
                file_exists: !game_exe.is_empty(),
                steam: Some(SteamGameInfo {
                    app_id: appid.clone(),
                    install_dir: installdir.clone(),
                    exe_path: game_exe.clone(),
                    cover_path: String::new(),
                    steam_dir: steam_dir.to_string_lossy().to_string(),
                }),
                multi_exe,
                is_primary,
            });
        }

        done += 1;
        let _ = app.emit("scan-progress", serde_json::json!({
            "phase": "icons",
            "found": total,
            "done": done,
            "total": total,
            "current": name,
        }));
    }

    Ok(ScanResult { results, total })
}

