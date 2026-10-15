<script setup lang="ts">
import { onMounted, ref, computed, type Component } from 'vue'
import TopNav from '@/components/nav/TopNav.vue'
import SideNav from '@/components/nav/SideNav.vue'
import FooterNav from '@/components/nav/FooterNav.vue'
import VerticalCardView from '@/components/views/VerticalCardView.vue'
import HorizontalCardView from '@/components/views/HorizontalCardView.vue'
import IconGridView from '@/components/views/IconGridView.vue'
import TableView from '@/components/views/TableView.vue'
import WaterfallView from '@/components/views/WaterfallView.vue'
import EntryEditor from '@/components/dialogs/EntryEditor.vue'
import EnvironmentManager from '@/components/dialogs/EnvironmentManager.vue'
import CategoryManager from '@/components/dialogs/CategoryManager.vue'
import ScanDialog from '@/components/dialogs/ScanDialog.vue'
import BookmarkImportDialog from '@/components/dialogs/BookmarkImportDialog.vue'
import DeleteCategoryDialog from '@/components/dialogs/DeleteCategoryDialog.vue'
import SettingsDialog from '@/components/dialogs/SettingsDialog.vue'
import AboutDialog from '@/components/dialogs/AboutDialog.vue'
import BatchToolbar from '@/components/BatchToolbar.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import {
  state, initStore, getCurrentEntries, getCurrentCategory, getCategoryById, getSubCategories,
  updateCategory, initTheme, saveSettings, unlockHiddenCategories, deleteEntries,
} from '@/store'
import type { Entry } from '@/types'
import { useToast } from '@/composables/useToast'
import { useDialogs } from '@/composables/useDialogs'
import { usePathStatus } from '@/composables/usePathStatus'
import { useEntryLaunch } from '@/composables/useEntryLaunch'
import { useDragDrop } from '@/composables/useDragDrop'
import { useEntryContextMenu } from '@/composables/useEntryContextMenu'
import { toggleInSet } from '@/utils/toggleInSet'

// ─── 初始化 ───
const loading = ref(true)
const { toastMsg, toastVisible, showToast } = useToast()

// ─── 对话框状态（集中在 useDialogs）───
const dialogs = useDialogs()
const {
  showEntryEditor, editingEntry, editingEntryType,
  showEnvManager, showCatManager, showScanDialog, showBookmarkDialog,
  showSettingsDialog, showAboutDialog,
  deleteCatId, catManagerEditId, catManagerNewParentId,
  showSteamChoiceDialog, steamChoiceEntry,
} = dialogs

const currentEntries = computed(() => getCurrentEntries())
const currentCategory = computed(() => getCurrentCategory())

// ─── 路径状态检查 ───
const { pathStatusMap, refreshPathStatus } = usePathStatus(() => currentEntries.value)

// ─── 条目启动（含 Steam 双方式选择）───
const { handleLaunch, launchSteamGame, openInExplorer, hasFilePath } = useEntryLaunch({
  showSteamChoiceDialog, steamChoiceEntry,
})

// ─── 拖拽收录 ───
const { dropOverlayVisible, setupDragDrop } = useDragDrop(() => refreshPathStatus())

// ─── 右键菜单 ───
async function handleDeleteEntry(entry: Entry) {
  try {
    await deleteEntries([entry.id])
    showToast(`已删除: ${entry.name}`)
    refreshPathStatus()
  } catch {
    showToast('删除失败')
  }
}

const { contextMenu, openEntryContextMenu, closeContextMenu } = useEntryContextMenu({
  onLaunch: handleLaunch,
  onEdit: (entry) => dialogs.openEntryEditor(entry),
  onDelete: handleDeleteEntry,
  hasFilePath,
  onOpenInExplorer: openInExplorer,
})

// ─── 侧导航折叠 ───

/** 当前一级分类是否有子分类（无子分类时隐藏侧导航） */
const hasSubCategories = computed(() => {
  let cat = state.currentCategoryId ? getCategoryById(state.currentCategoryId) : undefined
  while (cat && cat.parent_id) {
    cat = getCategoryById(cat.parent_id)
  }
  if (!cat) return false
  return getSubCategories(cat.id).length > 0
})

/** 侧导航当前是否折叠（隐藏），来自已持久化配置 */
const sidebarCollapsed = computed(() => state.config.sidebar_collapsed)

/** 折叠/展开侧导航：点击立即持久化 */
async function toggleSidebar() {
  const next = !sidebarCollapsed.value
  state.config.sidebar_collapsed = next
  try {
    await saveSettings({ sidebar_collapsed: next })
  } catch (e) {
    console.error('保存侧导航折叠状态失败', e)
  }
}

/** 侧导航是否应显示（有子分类且未折叠） */
const showSidebar = computed(() => hasSubCategories.value && !sidebarCollapsed.value)

// ─── 视图分发 ───

const currentViewMode = computed(() => {
  return currentCategory.value?.view_mode || 'icon_grid'
})

const viewComponent = computed<Component>(() => {
  switch (currentViewMode.value) {
    case 'vertical_card': return VerticalCardView
    case 'horizontal_card': return HorizontalCardView
    case 'icon_grid': return IconGridView
    case 'waterfall': return WaterfallView
    default: return TableView
  }
})

const viewBindings = computed(() => ({
  entries: currentEntries.value,
  pathStatus: pathStatusMap.value,
  batchMode: state.batchMode,
  selectedIds: state.selectedIds,
}))

// 切换视图模式（底部右侧）
function switchViewMode(mode: string) {
  if (currentCategory.value) {
    updateCategory(currentCategory.value.id, { view_mode: mode as any })
  }
}

// ─── 其余回调 ───

function onEntrySaved() {
  showEntryEditor.value = false
  refreshPathStatus()
}

function handleSearch(q: string) {
  state.searchQuery = q
}

function handleBookmarksImported(count: number) {
  showToast(`已导入 ${count} 个书签`)
  refreshPathStatus()
}

function handleCategoryDeleted() {
  showToast('分类已删除')
  refreshPathStatus()
}

onMounted(async () => {
  try {
    await initStore()
    loading.value = false
    initTheme() // 应用已保存的明暗主题（system 模式监听系统切换）
    refreshPathStatus()
    setupDragDrop()
  } catch (e) {
    console.error('初始化失败', e)
    loading.value = false
  }
})
</script>

<template>
  <div class="app-layout" v-if="!loading" @contextmenu.prevent>
    <!-- 顶部导航栏 -->
    <header class="app-header">
      <TopNav
        @search="handleSearch"
        @open-settings="dialogs.openAboutAndUnlock(unlockHiddenCategories)"
      />
    </header>

    <!-- 主体区域 -->
    <div class="app-body">
      <!-- 侧导航（一级分类无子分类或已折叠时隐藏） -->
      <aside class="app-sidebar" v-if="showSidebar">
        <SideNav
          @edit-category="dialogs.openCatManager($event)"
          @add-sub-category="dialogs.openCatManager(undefined, $event)"
          @delete-category="deleteCatId = $event"
        />
      </aside>

      <!-- 侧导航折叠/展开按钮（常驻，位于侧导航右边线上垂直居中） -->
      <button
        v-if="hasSubCategories"
        class="sidebar-toggle"
        :class="{ 'sidebar-collapsed': !showSidebar }"
        :title="showSidebar ? '隐藏侧导航' : '显示侧导航'"
        @click="toggleSidebar"
      >
        <span>{{ showSidebar ? '◀' : '▶' }}</span>
      </button>

      <!-- 内容区 -->
      <main class="app-content">
        <!-- 批量工具栏 -->
        <BatchToolbar v-if="state.batchMode" />

        <!-- 空状态 -->
        <div class="empty-state" v-if="currentEntries.length === 0 && !state.searchQuery">
          <div class="empty-icon">📂</div>
          <div class="empty-text">该分类下暂无条目</div>
          <div style="margin-top: 12px;">
            <button class="btn btn-primary" @click="dialogs.openEntryEditor(null)">+ 添加项目</button>
          </div>
        </div>

        <!-- 搜索空结果 -->
        <div class="empty-state" v-else-if="currentEntries.length === 0 && state.searchQuery">
          <div class="empty-icon">🔍</div>
          <div class="empty-text">未找到匹配 "{{ state.searchQuery }}" 的条目</div>
        </div>

        <!-- 动态视图组件 -->
        <component :is="viewComponent"
          v-else
          v-bind="viewBindings"
          @launch="handleLaunch"
          @edit="dialogs.openEntryEditor($event)"
          @contextmenu="openEntryContextMenu"
          @toggle-select="(id: string) => toggleInSet(state.selectedIds, id)"
        />
      </main>
    </div>

    <!-- 底部菜单栏（常驻，不受侧导航显隐影响） -->
    <footer class="app-footer">
      <FooterNav
        :view-mode="currentViewMode"
        :has-category="!!currentCategory"
        :batch-mode="state.batchMode"
        @switch-view="switchViewMode"
        @add-entry="dialogs.openEntryEditor(null)"
        @manage-categories="dialogs.openCatManager()"
        @manage-environments="showEnvManager = true"
        @scan-directory="showScanDialog = true"
        @import-bookmarks="showBookmarkDialog = true"
        @open-settings="showSettingsDialog = true"
      />
    </footer>

    <!-- 右键菜单 -->
    <ContextMenu
      v-if="contextMenu"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :items="contextMenu.items"
      @close="closeContextMenu"
    />

    <!-- 对话框 -->
    <EntryEditor
      v-if="showEntryEditor"
      :entry="editingEntry"
      :entry-type="editingEntryType"
      :current-category-id="state.currentCategoryId || ''"
      @close="showEntryEditor = false"
      @saved="onEntrySaved"
    />
    <EnvironmentManager
      v-if="showEnvManager"
      @close="showEnvManager = false"
    />
    <CategoryManager
      v-if="showCatManager"
      :initial-edit-id="catManagerEditId"
      :initial-new-parent-id="catManagerNewParentId"
      @close="dialogs.closeCatManager()"
    />
    <ScanDialog
      v-if="showScanDialog"
      @close="showScanDialog = false"
      @scanned="refreshPathStatus"
    />
    <!-- Steam 启动方式选择弹窗 -->
    <div class="dialog-overlay" v-if="showSteamChoiceDialog" @click.self="showSteamChoiceDialog = false">
      <div class="dialog steam-choice-dialog">
        <div class="dialog-header">
          <span>启动方式</span>
          <button class="btn btn-icon" @click="showSteamChoiceDialog = false">✕</button>
        </div>
        <div class="dialog-body">
          <p class="steam-choice-title">{{ steamChoiceEntry?.name }}</p>
          <p class="steam-choice-tip">请选择该 Steam 游戏的启动方式</p>
          <div class="steam-choice-buttons">
            <button class="btn steam-choice-btn" @click="launchSteamGame('steam')">
              <span class="steam-choice-btn-title">Steam 启动</span>
              <span class="steam-choice-btn-desc">通过 steam:// 协议唤起 Steam 客户端启动（推荐，兼容 DRM）</span>
            </button>
            <button class="btn steam-choice-btn" @click="launchSteamGame('exe')">
              <span class="steam-choice-btn-title">直接启动 exe</span>
              <span class="steam-choice-btn-desc">直接运行游戏主程序，部分依赖 Steam 环境的游戏可能失败</span>
            </button>
          </div>
        </div>
      </div>
    </div>
    <BookmarkImportDialog
      v-if="showBookmarkDialog"
      @close="showBookmarkDialog = false"
      @imported="handleBookmarksImported"
    />
    <DeleteCategoryDialog
      v-if="deleteCatId"
      :category-id="deleteCatId"
      @close="deleteCatId = null"
      @deleted="handleCategoryDeleted"
    />
    <SettingsDialog
      v-if="showSettingsDialog"
      @close="showSettingsDialog = false"
    />
    <AboutDialog
      v-if="showAboutDialog"
      @close="showAboutDialog = false"
    />

    <!-- 拖拽添加遮罩 -->
    <div class="drop-overlay" v-if="dropOverlayVisible">
      <div class="drop-overlay-box">
        <div class="drop-overlay-icon">⬇</div>
        <div class="drop-overlay-text">松开以添加</div>
      </div>
    </div>

    <!-- Toast -->
    <div class="toast" v-if="toastVisible">{{ toastMsg }}</div>
  </div>

  <!-- 加载中 -->
  <div class="loading-screen" v-else>
    <div class="loading-spinner"></div>
    <div class="loading-text">正在加载...</div>
  </div>
</template>

<style scoped>
.app-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-header {
  flex-shrink: 0;
}

.app-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

.app-sidebar {
  width: 180px;
  flex-shrink: 0;
  background: var(--gradient-nav);
  border-right: 1px solid var(--color-border);
  overflow-y: auto;
}

/* 侧导航折叠/展开按钮：贴在侧导航右边线，垂直居中 */
.sidebar-toggle {
  position: absolute;
  top: 50%;
  left: 180px;
  transform: translateY(-50%);
  z-index: 30;
  width: 18px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 1px solid var(--color-border);
  border-left: none;
  border-radius: 0 6px 6px 0;
  background: var(--color-bg-card, var(--color-bg));
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background var(--transition), color var(--transition);
  font-size: 10px;
  line-height: 1;
}

.sidebar-toggle:hover {
  background: var(--color-primary);
  color: #fff;
}

/* 已折叠时贴左边缘 */
.sidebar-toggle.sidebar-collapsed {
  left: 0;
  border-left: 1px solid var(--color-border);
  border-radius: 0 6px 6px 0;
}

.app-footer {
  flex-shrink: 0;
}

.app-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  /* 容器查询：子级按内容区实际宽度响应（自动扣除侧导航宽度） */
  container-type: inline-size;
}

.loading-screen {
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
}

/* Steam 启动方式选择弹窗 */
.steam-choice-dialog {
  width: 420px;
}

.steam-choice-title {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 4px;
}

.steam-choice-tip {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin: 0 0 16px;
}

.steam-choice-buttons {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.steam-choice-btn {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 12px 16px;
  text-align: left;
}

.steam-choice-btn:hover {
  border-color: var(--color-primary);
  background: var(--color-primary-bg);
}

.steam-choice-btn-title {
  font-size: 13px;
  font-weight: 600;
}

.steam-choice-btn-desc {
  font-size: 11px;
  color: var(--color-text-tertiary);
  font-weight: 400;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--color-border);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-text {
  color: var(--color-text-tertiary);
  font-size: 14px;
}

.drop-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(79, 131, 245, 0.12);
  border: 2px dashed var(--color-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.drop-overlay-box {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 28px 48px;
  text-align: center;
  box-shadow: var(--shadow-md);
}

.drop-overlay-icon {
  font-size: 32px;
  color: var(--color-primary);
  margin-bottom: 8px;
}

.drop-overlay-text {
  font-size: 15px;
  color: var(--color-text);
  font-weight: 500;
}
</style>
