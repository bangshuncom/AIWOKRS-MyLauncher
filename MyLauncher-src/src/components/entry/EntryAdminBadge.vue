<script setup lang="ts">
// 「以管理员身份运行」标记。卡片视图绝对定位于右上角，表格视图行内显示在名称后。
withDefaults(
  defineProps<{
    /** 绝对定位（卡片类视图）或行内（表格视图） */
    absolute?: boolean
    /** 绝对定位时的顶部偏移 */
    top?: number
    /** 绝对定位时的右侧偏移 */
    right?: number
    /** 字号（px） */
    size?: number
  }>(),
  { absolute: true, top: 8, right: 32, size: 16 },
)
</script>

<template>
  <div
    class="entry-admin-badge"
    :class="{ 'entry-admin-badge-absolute': absolute, 'entry-admin-badge-inline': !absolute }"
    :style="absolute ? { top: top + 'px', right: right + 'px', fontSize: size + 'px' } : { fontSize: size + 'px' }"
    title="以管理员身份运行"
  >
    🛡
  </div>
</template>
