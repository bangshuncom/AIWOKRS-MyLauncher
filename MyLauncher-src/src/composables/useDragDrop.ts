// ════════════════════════════════════════════
// 窗口文件拖拽收录：拖入 exe / lnk / 任意文件，自动识别并添加
// ════════════════════════════════════════════

import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import * as api from '@/api'
import { state, addEntry, createEntryDraft } from '@/store'
import { useToast } from '@/composables/useToast'

export function useDragDrop(onAdded?: () => void) {
  const { showToast } = useToast()
  const dropOverlayVisible = ref(false)
  let dropHideTimer: ReturnType<typeof setTimeout> | null = null

  async function handleDroppedFiles(paths: string[]) {
    if (paths.length === 0) return

    let ok = 0
    for (const p of paths) {
      try {
        const draft = createEntryDraft(state.currentCategoryId || '', 'program')

        if (/\.(exe|bat|cmd|lnk)$/i.test(p)) {
          if (/\.lnk$/i.test(p)) {
            const lnk = await api.resolve_lnk(p)
            draft.name = lnk.name
            draft.type = 'program'
            draft.relative_path = lnk.relative_path
            draft.absolute_paths = lnk.target_path ? { [state.currentEnvId]: lnk.target_path } : {}
            draft.path_mode = (lnk.suggested_path_mode === 'relative' ? 'relative' : 'absolute') as 'relative' | 'absolute'
            draft.launch_args = lnk.arguments || ''
            draft.working_directory = lnk.working_directory || ''
            draft.window_style = (lnk.window_style as 'normal' | 'maximized' | 'minimized') || 'normal'
            draft.notes = lnk.description || ''
            if (lnk.icon_path) draft.icon = { type: 'extracted', source: lnk.icon_path }
          } else {
            const info = await api.get_exe_info(p)
            draft.name = info.name || p.split(/[\\/]/).pop()?.replace(/\.(exe|bat|cmd)$/i, '') || '未命名'
            draft.type = 'program'
            draft.relative_path = info.relative_path
            draft.absolute_paths = info.absolute_path ? { [state.currentEnvId]: info.absolute_path } : {}
            draft.path_mode = (info.suggested_path_mode === 'relative' ? 'relative' : 'absolute') as 'relative' | 'absolute'
            if (info.icon_path) draft.icon = { type: 'extracted', source: info.icon_path }
          }
        } else {
          const info = await api.get_file_info(p)
          if (!info.file_exists) {
            console.error('文件不存在', p)
            continue
          }
          draft.name = info.name
          draft.type = info.entry_type as 'folder' | 'file'
          draft.relative_path = info.relative_path
          draft.absolute_paths = info.absolute_path ? { [state.currentEnvId]: info.absolute_path } : {}
          draft.path_mode = (info.suggested_path_mode === 'relative' ? 'relative' : 'absolute') as 'relative' | 'absolute'
          if (info.icon_path) draft.icon = { type: 'extracted', source: info.icon_path }
        }

        await addEntry(draft)
        ok++
      } catch (e) {
        console.error('拖拽添加失败', p, e)
      }
    }

    showToast(ok > 0 ? `已添加 ${ok} 个条目` : '添加失败')
    if (ok > 0) onAdded?.()
  }

  /** 监听 Tauri 窗口拖拽事件 */
  function setupDragDrop() {
    const win = getCurrentWindow()
    win.onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        dropOverlayVisible.value = true
        // 每次 over 刷新兜底定时器
        if (dropHideTimer) clearTimeout(dropHideTimer)
        dropHideTimer = setTimeout(() => {
          dropOverlayVisible.value = false
        }, 800)
      } else if (event.payload.type === 'leave') {
        if (dropHideTimer) clearTimeout(dropHideTimer)
        dropOverlayVisible.value = false
      } else if (event.payload.type === 'drop') {
        if (dropHideTimer) clearTimeout(dropHideTimer)
        dropOverlayVisible.value = false
        handleDroppedFiles(event.payload.paths)
      }
    })
  }

  return { dropOverlayVisible, handleDroppedFiles, setupDragDrop }
}
