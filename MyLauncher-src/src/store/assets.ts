// ═══════════════════════════════════════════
// 资源文件：路径解析与过期清理
// ═══════════════════════════════════════════

import { convertFileSrc } from '@tauri-apps/api/core'
import * as api from '@/api'
import type { Entry } from '@/types'
import { state } from './state'

function collectEntryAssetSources(entries: Entry[]): string[] {
  const sources: string[] = []
  for (const e of entries) {
    if (e.icon?.source) sources.push(e.icon.source)
    if (e.cover?.source) sources.push(e.cover.source)
  }
  return sources
}

// 删除条目后联动清理不再被引用的图标/封面文件（后端移入回收站，可还原）；
// domain 缓存图标仍被其他链接共用时由后端引用判断自动跳过。
// 清理失败不影响删除主流程（仅打印警告）。
export async function cleanupEntriesAssets(deletedEntries: Entry[]) {
  const sources = collectEntryAssetSources(deletedEntries)
  if (sources.length === 0) return
  try {
    await api.cleanup_deleted_assets(sources)
  } catch (e) {
    console.warn('删除后资源清理失败:', e)
  }
}

// ─── 资源路径解析 ───

/** 全局单调递增的图标版本计数器（iconVersions 各项共用） */
let iconVersionCounter = 0

/** 标记某图标文件已更新：bump 版本号，resolveAssetUrl 拼上 ?v= 强制 WebView 重载 */
export function bumpIconVersion(path: string) {
  if (!path) return
  iconVersionCounter++
  state.iconVersions.set(path, iconVersionCounter)
}

/**
 * 将后端返回的相对路径（如 "assets/icons/extracted/foo.png"）转为 WebView 可访问的 URL。
 * - 如果传入的是 http(s) URL，直接返回。
 * - 如果 exeDir 未就绪，返回原始路径（降级处理）。
 * - 若该路径被 bump 过版本（重置图标后内容更新），拼上 ?v= 查询参数绕过 WebView 缓存
 *   （Tauri asset 协议只取 URL 的 path 部分读文件，查询参数不影响解析）。
 */
export function resolveAssetUrl(path: string): string {
  if (!path) return ''
  // 已经是 URL 的直接返回
  if (/^https?:\/\//i.test(path)) return path
  // 已经是 convertFileSrc 格式
  if (path.startsWith('asset://') || path.startsWith('tauri://')) return path
  // 版本参数（asset 协议忽略查询串，仅用于绕过 WebView 图片缓存）
  const version = state.iconVersions.get(path)
  const versionQuery = version ? `?v=${version}` : ''
  // 绝对路径直接转换
  if (/^[A-Za-z]:[\\/]/.test(path)) {
    return convertFileSrc(path) + versionQuery
  }
  // 相对路径：拼接 exeDir 后转换
  if (state.exeDir) {
    const sep = state.exeDir.endsWith('\\') || state.exeDir.endsWith('/') ? '' : '/'
    const abs = state.exeDir + sep + path.replace(/^\.\//, '')
    return convertFileSrc(abs) + versionQuery
  }
  return path + versionQuery
}
