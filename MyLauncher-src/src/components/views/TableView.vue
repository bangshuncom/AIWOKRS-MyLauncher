<script setup lang="ts">
import type { Entry } from '@/types'
import { getCategoryById } from '@/store'
import { useEntryView, type EntryViewProps, type EntryViewEmits } from '@/composables/useEntryView'
import { formatRelativeTime } from '@/utils/format'
import EntryCheckbox from '@/components/entry/EntryCheckbox.vue'
import EntryAdminBadge from '@/components/entry/EntryAdminBadge.vue'
import EntryIcon from '@/components/entry/EntryIcon.vue'

const props = defineProps<EntryViewProps>()
const emit = defineEmits<EntryViewEmits>()
const { onHoverCheck, onContextMenu, pathExists, onDoubleClick, hoverTip } = useEntryView(props, emit)

function getCategoryName(catId: string): string {
  const cat = getCategoryById(catId)
  return cat?.name || '未分类'
}

function onRowClick(entry: Entry) {
  if (props.batchMode) emit('toggle-select', entry.id)
}
</script>

<template>
  <div class="table-view">
    <table class="entry-table">
      <thead>
        <tr>
          <th class="col-check"></th>
          <th class="col-icon"></th>
          <th class="col-name">名称</th>
          <th class="col-category">分类</th>
          <th class="col-last-used">最后使用</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="entry in entries"
          :key="entry.id"
          class="table-row entry-cell"
          :class="{
            'path-invalid': !pathExists(entry),
            'selected': batchMode && selectedIds.has(entry.id),
          }"
          :title="hoverTip(entry)"
          @dblclick="onDoubleClick(entry)"
          @click="onRowClick(entry)"
          @contextmenu="onContextMenu(entry, $event)"
        >
          <td class="col-check">
            <EntryCheckbox
              :checked="batchMode && selectedIds.has(entry.id)"
              :hover-only="!batchMode"
              :absolute="false"
              :size="16"
              @click="onHoverCheck($event, entry)"
            />
          </td>
          <td class="col-icon">
            <EntryIcon
              :entry="entry"
              :size="20"
              :fallback-size="11"
              show-initial
              fallback-class="entry-fallback-badge"
            />
          </td>
          <td class="col-name">
            <span class="row-name" :title="entry.name">{{ entry.name }}</span>
            <EntryAdminBadge v-if="entry.run_as_admin" :absolute="false" :size="11" />
          </td>
          <td class="col-category">{{ getCategoryName(entry.category_id) }}</td>
          <td class="col-last-used">{{ formatRelativeTime(entry.last_used) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.table-view {
  /* 名称列自适应剩余宽度并截断，表格总宽不超容器 → 不出现横向滚动条 */
  overflow-x: hidden;
  padding-bottom: 20px;
}

.entry-table {
  width: 100%;
  table-layout: fixed;
  border-collapse: collapse;
  font-size: 13px;
}

.entry-table th {
  text-align: left;
  padding: 8px 12px;
  background: var(--color-bg-sidebar);
  border-bottom: 1px solid var(--color-border);
  font-weight: 600;
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.entry-table th:first-child {
  border-radius: var(--radius-sm) 0 0 0;
}

.entry-table th:last-child {
  border-radius: 0 var(--radius-sm) 0 0;
}

.entry-table td {
  padding: 8px 12px;
  border-bottom: 1px solid var(--color-border-light);
  white-space: nowrap;
}

.table-row {
  cursor: pointer;
  transition: background var(--transition);
}

.table-row:hover {
  background: var(--color-bg-hover);
}

.table-row.path-invalid {
  background: var(--color-danger-bg);
}

.table-row.path-invalid:hover {
  background: var(--color-path-invalid);
}

.table-row.selected {
  background: var(--color-primary-bg);
}

.col-check {
  width: 36px;
  text-align: center;
}

.col-icon {
  width: 40px;
}

.col-name {
  /* 占据剩余宽度：其他列内容固定窄，此列吃掉全部剩余空间 */
  width: auto;
}

.row-name {
  font-weight: 500;
  /* 名称过长截断省略，悬停 title 看全名 */
  display: inline-block;
  max-width: calc(100% - 20px);
  vertical-align: bottom;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.col-category {
  color: var(--color-text-secondary);
  /* 固定窄列：让名称列独占剩余宽度（table-layout:fixed 下未设宽的列会平分剩余空间） */
  width: 110px;
  /* 分类名过长同样截断保底（正常不会太长） */
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-last-used {
  color: var(--color-text-tertiary);
  font-size: 12px;
  /* 固定窄列：仅显示“3分钟前/2天前/日期”短文本 */
  width: 90px;
}
</style>
