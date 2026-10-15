// ═══════════════════════════════════════════
// 环境：增删改与切换
// ═══════════════════════════════════════════

import * as api from '@/api'
import type { Environment } from '@/types'
import { state } from './state'
import { cleanupEntriesAssets } from './assets'

// ─── Actions: 环境 ───

export async function addEnvironment(env: Omit<Environment, 'id'>): Promise<string> {
  const id = `env_${String(state.environments.length + 1).padStart(3, '0')}_${Date.now()}`
  const newEnv: Environment = { ...env, id }
  state.environments.push(newEnv)
  await api.save_environments(state.environments)
  return id
}

export async function updateEnvironment(id: string, updates: Partial<Environment>) {
  const idx = state.environments.findIndex(e => e.id === id)
  if (idx >= 0) {
    state.environments[idx] = { ...state.environments[idx], ...updates }
    await api.save_environments(state.environments)
  }
}

export async function deleteEnvironment(id: string, deleteEntries: boolean) {
  if (deleteEntries) {
    // 收集被删条目并联动清理资源
    const deletedEntries = state.entries.filter(e => {
      return e.path_mode === 'absolute' && e.absolute_paths && id in e.absolute_paths
    })
    state.entries = state.entries.filter(e => {
      // 删除该环境下的绝对路径条目
      if (e.path_mode === 'absolute' && e.absolute_paths && id in e.absolute_paths) {
        return false
      }
      return true
    })
    await cleanupEntriesAssets(deletedEntries)
  } else {
    // 仅从条目的 absolute_paths 中移除该环境
    state.entries.forEach(e => {
      if (e.absolute_paths && id in e.absolute_paths) {
        delete e.absolute_paths[id]
      }
    })
  }

  state.environments = state.environments.filter(e => e.id !== id)
  await api.save_environments(state.environments)
  await api.save_entries(state.entries)

  // 切换到第一个环境
  if (state.currentEnvId === id && state.environments.length > 0) {
    state.currentEnvId = state.environments[0].id
    state.environments[0].is_current = true
    await api.save_environments(state.environments)
  }
}

export async function switchEnvironment(id: string) {
  state.environments.forEach(e => {
    e.is_current = e.id === id
  })
  state.currentEnvId = id
  await api.save_environments(state.environments)
}
