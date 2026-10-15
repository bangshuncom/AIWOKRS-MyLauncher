<script setup lang="ts">
import { useEntryView, type EntryViewProps, type EntryViewEmits } from '@/composables/useEntryView'
import EntryCheckbox from '@/components/entry/EntryCheckbox.vue'
import EntryAdminBadge from '@/components/entry/EntryAdminBadge.vue'
import EntryIcon from '@/components/entry/EntryIcon.vue'

const props = defineProps<EntryViewProps>()
const emit = defineEmits<EntryViewEmits>()
const { onHoverCheck, onContextMenu, pathExists, onDoubleClick, hoverTip } = useEntryView(props, emit)
</script>

<template>
  <div class="icon-grid">
    <div
      v-for="entry in entries"
      :key="entry.id"
      class="icon-cell entry-cell"
      :class="{
        'path-invalid': !pathExists(entry),
        'selected': batchMode && selectedIds.has(entry.id),
      }"
      :title="hoverTip(entry)"
      @dblclick="onDoubleClick(entry)"
      @contextmenu="onContextMenu(entry, $event)"
    >
      <!-- hover 勾选框（始终显示在右上角，非多选时 hover 可见） -->
      <EntryCheckbox
        :checked="batchMode && selectedIds.has(entry.id)"
        :hover-only="!batchMode"
        :offset="4"
        :size="16"
        @click="onHoverCheck($event, entry)"
      />

      <!-- 管理员标识 -->
      <EntryAdminBadge v-if="entry.run_as_admin" :top="4" :right="24" :size="12" />

      <!-- 图标 -->
      <div class="icon-wrapper">
        <EntryIcon :entry="entry" :size="48" :emoji-size="38" :fallback-size="20" show-initial />
      </div>

      <!-- 名称 -->
      <span class="icon-name">{{ entry.name }}</span>
      <span v-if="!pathExists(entry)" class="icon-status">⚠</span>
    </div>
  </div>
</template>

<style scoped>
.icon-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
  gap: 12px;
  padding: 4px 0 20px;
}

.icon-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 12px 8px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition);
  position: relative;
  text-align: center;
}

.icon-cell:hover {
  background: var(--color-bg-hover);
}

.icon-cell.path-invalid .icon-wrapper {
  filter: grayscale(1) opacity(0.5);
}

.icon-cell.selected {
  background: var(--color-primary-bg);
  box-shadow: 0 0 0 2px var(--color-primary);
}

.icon-wrapper {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-name {
  font-size: 12px;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.icon-status {
  font-size: 10px;
  color: var(--color-danger);
}
</style>
