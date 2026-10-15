// ════════════════════════════════════════════
// 条目启动：网址 / 程序 / Steam 游戏
// Steam 支持两种启动方式（steam:// 协议 / 直接跑 exe），由调用方传入的弹窗状态驱动
// ════════════════════════════════════════════

import type { Ref } from 'vue'
import * as api from '@/api'
import { state, updateLastUsed } from '@/store'
import type { Entry } from '@/types'
import { useToast } from '@/composables/useToast'

export interface SteamChoiceRefs {
  showSteamChoiceDialog: Ref<boolean>
  steamChoiceEntry: Ref<Entry | null>
}

/** 有文件路径的类型（网址/系统功能/APPX 不显示「目录」；Steam 游戏定位到游戏 exe） */
export function hasFilePath(entry: Entry): boolean {
  return entry.type === 'program' || entry.type === 'folder' || entry.type === 'file'
    || (entry.type === 'steam' && !!Object.values(entry.absolute_paths)[0])
}

export function useEntryLaunch(steam: SteamChoiceRefs) {
  const { showToast } = useToast()

  async function handleLaunch(entry: Entry) {
    // Steam 游戏条目：弹窗让用户选择启动方式（Steam 协议 / 直接启动 exe）
    if (entry.type === 'steam') {
      steam.steamChoiceEntry.value = entry
      steam.showSteamChoiceDialog.value = true
      return
    }

    if (entry.type === 'url') {
      const success = await api.launch_program(entry, state.currentEnvId)
      if (success) {
        await updateLastUsed(entry.id)
        showToast(`已打开: ${entry.name}`)
      }
      return
    }

    const pathInfo = await api.resolve_path(entry, state.currentEnvId)
    if (!pathInfo.exists) {
      showToast(`路径失效: ${entry.name}`)
      return
    }
    const success = await api.launch_program(entry, state.currentEnvId)
    if (success) {
      await updateLastUsed(entry.id)
      showToast(`已启动: ${entry.name}`)
    } else {
      showToast(`启动失败: ${entry.name}`)
    }
  }

  /** 执行 Steam 游戏启动（按用户选择的方式） */
  async function launchSteamGame(mode: 'steam' | 'exe') {
    const entry = steam.steamChoiceEntry.value
    steam.showSteamChoiceDialog.value = false
    if (!entry) return

    if (mode === 'steam') {
      // 直接用原条目（url = steam://rungameid/{appid}）启动
      const success = await api.launch_program(entry, state.currentEnvId)
      if (success) {
        await updateLastUsed(entry.id)
        showToast(`已启动: ${entry.name}`)
      } else {
        showToast(`启动失败: ${entry.name}`)
      }
      return
    }

    // exe 方式：条目 absolute_paths 中保存了游戏 exe 路径（导入时存入），
    // 构造临时 program 条目直接启动 exe
    const exePath = Object.values(entry.absolute_paths)[0] || ''
    if (!exePath) {
      showToast(`游戏路径未保存: ${entry.name}`)
      return
    }
    const proxyEntry: Entry = {
      ...entry,
      type: 'program',
      path_mode: 'absolute',
      relative_path: '',
      absolute_paths: { [state.currentEnvId]: exePath },
    }
    const success = await api.launch_program(proxyEntry, state.currentEnvId)
    if (success) {
      await updateLastUsed(entry.id)
      showToast(`已启动: ${entry.name}`)
    } else {
      showToast(`启动失败: ${entry.name}`)
    }
  }

  /** 打开条目所在目录（资源管理器定位并选中） */
  async function openInExplorer(entry: Entry) {
    try {
      await api.open_in_explorer(entry, state.currentEnvId)
    } catch (e) {
      showToast(`打开目录失败: ${e}`)
    }
  }

  return { handleLaunch, launchSteamGame, openInExplorer, hasFilePath }
}
