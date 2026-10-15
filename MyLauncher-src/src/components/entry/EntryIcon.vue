<script setup lang="ts">
// 条目图标三态渲染：图片 → emoji → 首字母兜底。
// 这段 v-if/v-else-if/v-else 原先在 5 个视图里重复了 7 次，统一到此处。
// 各视图的尺寸差异通过 size / emojiSize / fallbackSize 传入，保持原有视觉。
import { computed } from 'vue'
import type { Entry } from '@/types'
import { getIconPath, getIconEmoji } from '@/utils/entryDisplay'
import { onImgError, onImgLoad } from '@/composables/useImageFallback'

const props = withDefaults(
  defineProps<{
    entry: Entry
    /** 图标边长（px） */
    size?: number
    /** emoji 字号（px），默认取 size 的 90% */
    emojiSize?: number
    /** 兜底首字母字号（px），默认取 size 的 55% */
    fallbackSize?: number
    /** 兜底块内是否显示首字母（卡片封面显示，卡片底部留空占位） */
    showInitial?: boolean
    /** 兜底块附加类名，用于表格视图的带底色徽标样式（定义在 global.css） */
    fallbackClass?: string
  }>(),
  { size: 20, emojiSize: 0, fallbackSize: 0, showInitial: false, fallbackClass: '' },
)

const emojiPx = computed(() => props.emojiSize || Math.round(props.size * 0.9))
const fallbackPx = computed(() => props.fallbackSize || Math.round(props.size * 0.55))
</script>

<template>
  <img
    v-if="getIconPath(entry)"
    :src="getIconPath(entry)"
    class="entry-icon-img"
    :style="{ width: size + 'px', height: size + 'px' }"
    alt="icon"
    @error="onImgError"
    @load="onImgLoad"
  />
  <span
    v-else-if="getIconEmoji(entry)"
    class="entry-icon-emoji"
    :style="{ fontSize: emojiPx + 'px' }"
  >{{ getIconEmoji(entry) }}</span>
  <div
    v-else
    class="entry-icon-fallback"
    :class="fallbackClass"
    :style="{ width: size + 'px', height: size + 'px' }"
  >
    <span v-if="showInitial" :style="{ fontSize: fallbackPx + 'px' }">{{ entry.name.charAt(0) || '?' }}</span>
  </div>
</template>

<style scoped>
.entry-icon-img {
  object-fit: contain;
  flex-shrink: 0;
}

.entry-icon-emoji {
  line-height: 1;
  flex-shrink: 0;
}

.entry-icon-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
</style>
