<script setup lang="ts">
// 条目批量勾选框。五个视图原先各自复制一份 markup + CSS，统一到此处。
// 勾选框的 hover 显隐规则依赖父级卡片的 hover，故样式放在 global.css（.entry-cell）。
withDefaults(
  defineProps<{
    checked: boolean
    /** 非多选模式下只在悬停/选中时显示 */
    hoverOnly?: boolean
    /** 绝对定位（卡片类视图）还是行内（表格视图） */
    absolute?: boolean
    /** 绝对定位时的右上偏移量 */
    offset?: number
    /** 复选框边长 */
    size?: number
  }>(),
  { hoverOnly: false, absolute: true, offset: 8, size: 18 },
)

const emit = defineEmits<{ (e: 'click', ev: MouseEvent): void }>()
</script>

<template>
  <input
    type="checkbox"
    class="entry-checkbox"
    :class="{ 'entry-checkbox-hover': hoverOnly, 'entry-checkbox-absolute': absolute }"
    :style="absolute ? { top: offset + 'px', right: offset + 'px', width: size + 'px', height: size + 'px' } : { width: size + 'px', height: size + 'px' }"
    :checked="checked"
    @click.stop="emit('click', $event)"
  />
</template>
