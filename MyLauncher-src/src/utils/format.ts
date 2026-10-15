// ════════════════════════════════════════════
// 通用格式化函数
// ════════════════════════════════════════════

/**
 * 相对时间：刚刚 / N分钟前 / N小时前 / N天前 / 日期
 * 传入空串返回占位符「—」，解析失败时原样返回。
 */
export function formatRelativeTime(time: string): string {
  if (!time) return '—'
  try {
    const d = new Date(time)
    const diff = Date.now() - d.getTime()
    if (diff < 60000) return '刚刚'
    if (diff < 3600000) return Math.floor(diff / 60000) + '分钟前'
    if (diff < 86400000) return Math.floor(diff / 3600000) + '小时前'
    if (diff < 2592000000) return Math.floor(diff / 86400000) + '天前'
    return d.toLocaleDateString('zh-CN')
  } catch {
    return time
  }
}
