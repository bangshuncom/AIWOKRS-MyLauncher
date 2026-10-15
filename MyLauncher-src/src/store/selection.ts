// ═══════════════════════════════════════════
// 批量选择模式与选中集合
// ═══════════════════════════════════════════

import type { Entry } from '@/types'
import { state } from './state'

// ─── Actions: 批量操作 ───

export function toggleBatchMode() {
  state.batchMode = !state.batchMode
  if (!state.batchMode) {
    state.selectedIds.clear()
  }
}

export function selectAllCurrent(entries: Entry[]) {
  entries.forEach(e => state.selectedIds.add(e.id))
}

export function invertSelection(entries: Entry[]) {
  entries.forEach(e => {
    if (state.selectedIds.has(e.id)) {
      state.selectedIds.delete(e.id)
    } else {
      state.selectedIds.add(e.id)
    }
  })
}

export function clearSelection() {
  state.selectedIds.clear()
}
