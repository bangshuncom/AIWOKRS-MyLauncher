// ════════════════════════════════════════════
// 四视图组件共享逻辑
// ════════════════════════════════════════════

import type { Entry } from '@/types'
import { state } from '@/store'
import { getIconPath, getIconEmoji, getCoverPath, hoverTip } from '@/utils/entryDisplay'

export interface EntryViewProps {
  entries: Entry[]
  pathStatus: Map<string, boolean>
  batchMode: boolean
  selectedIds: Set<string>
}

export interface EntryViewEmits {
  (e: 'launch', entry: Entry): void
  (e: 'edit', entry: Entry): void
  (e: 'toggle-select', id: string): void
  (e: 'contextmenu', entry: Entry, ev: MouseEvent): void
}

export function useEntryView(props: EntryViewProps, emit: EntryViewEmits) {
  function onHoverCheck(e: Event, entry: Entry) {
    e.stopPropagation()
    if (!props.batchMode) state.batchMode = true
    emit('toggle-select', entry.id)
  }

  function onContextMenu(entry: Entry, e: MouseEvent) {
    emit('contextmenu', entry, e)
  }

  function pathExists(entry: Entry): boolean {
    // Steam：后端 resolve 已优先解析 exe 路径，前端同样按 pathStatus 判定
    if (entry.type === 'url' || entry.type === 'system' || entry.type === 'appx') return true
    // 特殊启动目标（系统功能/APPX 的 URI 或裸命令名）由后端恒判有效，前端兑底不标失效
    const t = entry.relative_path?.trim() || ''
    if (t.startsWith('shell:') || t.startsWith('ms-settings:')) return true
    if (t && !t.includes('\\') && !t.includes('/') && !t.includes(':')) return true
    return props.pathStatus.get(entry.id) ?? true
  }

  function onDoubleClick(entry: Entry) {
    if (props.batchMode) emit('toggle-select', entry.id)
    else emit('launch', entry)
  }

  return { getIconPath, getIconEmoji, getCoverPath, onHoverCheck, onContextMenu, pathExists, onDoubleClick, hoverTip }
}
