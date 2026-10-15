// ════════════════════════════════════════════
// 条目右键菜单：打开 / 编辑 / 目录 / 删除
// 菜单项由调用方注入动作，本模块只负责坐标与菜单项的装配
// ════════════════════════════════════════════

import { ref } from 'vue'
import type { Entry } from '@/types'
import type { MenuItem } from '@/components/ContextMenu.vue'

export interface EntryMenuActions {
  onLaunch: (entry: Entry) => void
  onEdit: (entry: Entry) => void
  onDelete: (entry: Entry) => void
  /** 是否有可定位的文件路径（决定「目录」项是否出现） */
  hasFilePath: (entry: Entry) => boolean
  onOpenInExplorer: (entry: Entry) => void
}

export function useEntryContextMenu(actions: EntryMenuActions) {
  const contextMenu = ref<{ x: number; y: number; items: MenuItem[] } | null>(null)

  function openEntryContextMenu(entry: Entry, e: MouseEvent) {
    e.preventDefault()
    // 阻止冒泡到 window，避免 ContextMenu 自身的 contextmenu 监听器立即关闭菜单
    e.stopPropagation()
    contextMenu.value = {
      x: e.clientX,
      y: e.clientY,
      items: [
        { label: '打开', icon: '▶️', action: () => actions.onLaunch(entry) },
        { label: '编辑', icon: '✏️', action: () => actions.onEdit(entry) },
        ...(actions.hasFilePath(entry)
          ? [{ label: '目录', icon: '📂', action: () => actions.onOpenInExplorer(entry) }]
          : []),
        { label: '删除', icon: '🗑', danger: true, action: () => actions.onDelete(entry) },
      ],
    }
  }

  function closeContextMenu() {
    contextMenu.value = null
  }

  return { contextMenu, openEntryContextMenu, closeContextMenu }
}
