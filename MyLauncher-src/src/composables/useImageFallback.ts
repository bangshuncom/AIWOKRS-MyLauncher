// ════════════════════════════════════════════
// 图片加载兜底：失败时隐藏，成功时恢复显示
// 原先在 9 个组件里逐字重复了 17 次，统一到这里
// ════════════════════════════════════════════

export function onImgError(e: Event) {
  ;(e.target as HTMLImageElement).style.display = 'none'
}

export function onImgLoad(e: Event) {
  ;(e.target as HTMLImageElement).style.display = ''
}
