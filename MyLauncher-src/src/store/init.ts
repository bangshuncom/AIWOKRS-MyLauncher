// ═══════════════════════════════════════════
// 初始化（放在单独文件，避免 state ← categories 循环依赖）
// ═══════════════════════════════════════════

import * as api from '@/api'
import { state } from './state'
import { getTopCategories, isCategoryHidden } from './categories'

// ─── 初始化 ───

export async function initStore() {
  await api.init_data_dir()
  const data = await api.load_all_data()
  state.config = data.config
  state.environments = data.environments
  state.categories = data.categories
  state.entries = data.entries

  // 获取 exe 所在目录（用于拼接相对路径 → convertFileSrc）
  try {
    state.exeDir = await api.get_exe_dir()
  } catch {
    state.exeDir = ''
  }

  // 设置当前环境
  const currentEnv = state.environments.find(e => e.is_current)
  state.currentEnvId = currentEnv?.id || state.environments[0]?.id || 'env_001'

  // 设置默认选中第一个可见的顶级分类（跳过隐藏分类；全隐藏时回退第一个顶级分类）
  const topCats = getTopCategories()
  const visibleTop = topCats.filter(c => !isCategoryHidden(c))
  const firstCat = visibleTop[0] || topCats[0]
  if (firstCat) {
    state.currentCategoryId = firstCat.id
  }

  // 更新 ID 计数器
  const maxId = state.entries.reduce((max, e) => {
    const match = e.id.match(/entry_(\d+)/)
    return match ? Math.max(max, parseInt(match[1])) : max
  }, 0)
  state.idCounter = maxId + 1

  state.loaded = true
}
