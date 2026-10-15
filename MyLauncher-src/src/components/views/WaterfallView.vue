<script setup lang="ts">
import { computed } from 'vue'
import { useEntryView, type EntryViewProps, type EntryViewEmits } from '@/composables/useEntryView'
import { onImgError, onImgLoad } from '@/composables/useImageFallback'
import { state } from '@/store'
import EntryCheckbox from '@/components/entry/EntryCheckbox.vue'
import EntryAdminBadge from '@/components/entry/EntryAdminBadge.vue'
import EntryIcon from '@/components/entry/EntryIcon.vue'

const props = defineProps<EntryViewProps>()
const emit = defineEmits<EntryViewEmits>()
const { getCoverPath, onHoverCheck, onContextMenu, pathExists, onDoubleClick } = useEntryView(props, emit)

/** 瀑布流封面最大高度（px），从配置读取，默认 480 */
const maxHeight = computed(() => {
  const h = state.config?.masonry_max_height
  return typeof h === 'number' && h > 0 ? h : 480
})
</script>

<template>
  <div class="waterfall-list responsive-cols">
    <div
      v-for="entry in entries"
      :key="entry.id"
      class="w-card entry-cell"
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
      <EntryAdminBadge v-if="entry.run_as_admin" />

      <!-- 封面图：按原图比例自适应宽度，最多限高 -->
      <div class="w-card-cover">
        <img
          v-if="getCoverPath(entry)"
          :src="getCoverPath(entry)"
          alt="cover"
          :style="{ maxHeight: maxHeight + 'px' }"
          @error="onImgError"
          @load="onImgLoad"
        />
        <div v-else class="w-card-cover-fallback">
          <EntryIcon :entry="entry" :size="48" :emoji-size="46" :fallback-size="36" show-initial />
        </div>
      </div>

      <!-- 底部信息 -->
      <div class="w-card-footer">
        <EntryIcon :entry="entry" :size="20" />
        <span class="w-card-name">{{ entry.name }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.waterfall-list {
  /* 分档固定列数，与竖向卡片一致（容器查询，按内容区实际宽度）：最小 2 列，最大 6 列 */
  column-count: var(--cols, 2);
  column-gap: 16px;
  padding-bottom: 20px;
}

.w-card {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition);
  position: relative;
  margin-bottom: 16px;
  break-inside: avoid;
}

.w-card:hover {
  box-shadow: 0 4px 12px var(--color-shadow-strong);
  transform: translateY(-2px);
}

.w-card.path-invalid {
  border-color: var(--color-danger);
}

.w-card.selected {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-primary);
}

.w-card-cover {
  width: 100%;
  background: var(--color-bg-sidebar);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.w-card-cover img {
  width: 100%;
  height: auto;
  object-fit: cover;
  display: block;
}

.w-card-cover-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  min-height: 120px;
}

.w-card-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
}

.w-card-name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
