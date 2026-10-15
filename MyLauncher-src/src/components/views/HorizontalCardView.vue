<script setup lang="ts">
import { useEntryView, type EntryViewProps, type EntryViewEmits } from '@/composables/useEntryView'
import { onImgError, onImgLoad } from '@/composables/useImageFallback'
import EntryCheckbox from '@/components/entry/EntryCheckbox.vue'
import EntryAdminBadge from '@/components/entry/EntryAdminBadge.vue'
import EntryIcon from '@/components/entry/EntryIcon.vue'

const props = defineProps<EntryViewProps>()
const emit = defineEmits<EntryViewEmits>()
const { getCoverPath, onHoverCheck, onContextMenu, pathExists, onDoubleClick } = useEntryView(props, emit)
</script>

<template>
  <div class="h-card-list">
    <div
      v-for="entry in entries"
      :key="entry.id"
      class="h-card entry-cell"
      :class="{
        'path-invalid': !pathExists(entry),
        'selected': batchMode && selectedIds.has(entry.id),
      }"
      @dblclick="onDoubleClick(entry)"
      @contextmenu="onContextMenu(entry, $event)"
    >
      <!-- hover 勾选框 -->
      <EntryCheckbox
        :checked="batchMode && selectedIds.has(entry.id)"
        :hover-only="!batchMode"
        @click="onHoverCheck($event, entry)"
      />

      <!-- 管理员标识 -->
      <EntryAdminBadge v-if="entry.run_as_admin" :top="4" :right="32" :size="14" />

      <!-- 封面图 -->
      <div class="h-card-cover">
        <img v-if="getCoverPath(entry)" :src="getCoverPath(entry)" alt="cover" @error="onImgError" @load="onImgLoad" />
        <div v-else class="h-card-cover-fallback">
          <EntryIcon :entry="entry" :size="36" :emoji-size="34" :fallback-size="28" show-initial />
        </div>
      </div>

      <!-- 右侧信息 -->
      <div class="h-card-info">
        <div class="h-card-header">
          <EntryIcon :entry="entry" :size="18" :emoji-size="17" />
          <span class="h-card-name">{{ entry.name }}</span>
        </div>
        <!-- 网址类型：标题下显示 URL -->
        <div v-if="entry.type === 'url' && entry.url" class="h-card-url">{{ entry.url }}</div>
        <div class="h-card-meta">
          <span v-if="entry.tags.length" class="h-card-tags">
            <span v-for="tag in entry.tags.slice(0, 3)" :key="tag" class="tag">{{ tag }}</span>
          </span>
          <span v-if="entry.notes" class="h-card-notes">{{ entry.notes }}</span>
          <span v-if="!pathExists(entry)" class="status-invalid">⚠ 路径失效</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.h-card-list {
  display: grid;
  /* 每列至少 320px，最多撑满容器；列数随容器宽度自适应，列宽始终一致 */
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 12px;
  padding-bottom: 20px;
}

.h-card {
  display: flex;
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition);
  position: relative;
  height: 80px;
  min-width: 0; /* 允许内容收缩，防止溢出 */
}

.h-card:hover {
  box-shadow: 0 2px 8px var(--color-shadow-strong);
  border-color: var(--color-primary-light);
}

.h-card.path-invalid {
  border-color: var(--color-danger);
}

.h-card.selected {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary);
}

.h-card-cover {
  width: 80px;
  height: 100%;
  background: var(--color-bg-sidebar);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.h-card-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.h-card-cover-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

.h-card-info {
  flex: 1;
  padding: 8px 14px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
  overflow: hidden;
}

.h-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.h-card-name {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.h-card-url {
  font-size: 11px;
  color: var(--color-primary);
  opacity: 0.85;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.h-card-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.h-card-tags {
  display: flex;
  gap: 4px;
}

.h-card-notes {
  font-size: 11px;
  color: var(--color-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-invalid {
  font-size: 11px;
  color: var(--color-danger);
}
</style>
