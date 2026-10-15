// ═══════════════════════════════════════════
// 设置保存与明暗主题
// ═══════════════════════════════════════════

import * as api from '@/api'
import type { AppConfig } from '@/types'
import { state } from './state'

// ─── Actions: 设置 ───

/** 保存设置（合并到 state.config 并持久化） */
export async function saveSettings(updates: Partial<AppConfig>) {
  state.config = { ...state.config, ...updates }
  await api.save_config(state.config)
}

// ─── 主题（明/暗/随系统） ───

/** 系统暗色偏好媒体查询（惰性创建，模块级单例） */
let systemDarkQuery: MediaQueryList | null = null

/** 当前系统是否暗色 */
function systemPrefersDark(): boolean {
  if (!systemDarkQuery) {
    systemDarkQuery = window.matchMedia('(prefers-color-scheme: dark)')
  }
  return systemDarkQuery.matches
}

/** 按主题设置给 html 根元素挂/摘 dark 类 */
function applyThemeClass(theme: 'light' | 'dark' | 'system') {
  const dark = theme === 'dark' || (theme === 'system' && systemPrefersDark())
  document.documentElement.classList.toggle('dark', dark)
}

/**
 * 切换主题并持久化；system 模式注册系统偏好变化监听（变化时即时切换）。
 * 应用启动时也应调用一次以应用已保存的主题。
 */
export async function setTheme(theme: 'light' | 'dark' | 'system') {
  state.config.theme = theme
  applyThemeClass(theme)
  await api.save_config(state.config)
}

/** 应用启动时初始化主题（应用配置并监听系统明暗变化，仅 system 模式生效） */
export function initTheme() {
  applyThemeClass(state.config.theme)
  if (!systemDarkQuery) {
    systemDarkQuery = window.matchMedia('(prefers-color-scheme: dark)')
    systemDarkQuery.addEventListener('change', () => {
      // 仅随系统模式下响应（用户手动指定明/暗时不跟随）
      if (state.config.theme === 'system') {
        applyThemeClass('system')
      }
    })
  }
}
