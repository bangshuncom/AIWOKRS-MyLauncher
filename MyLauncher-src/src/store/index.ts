// ════════════════════════════════════════════
// 全局状态管理（Vue 3 reactive，未引入 Pinia）
//
// 本文件只做统一出口：按领域拆分后的实现见同目录各模块。
// 外部一律 `import { xxx } from '@/store'`，无需关心内部分文件。
// ════════════════════════════════════════════

export * from './state'
export * from './init'
export * from './assets'
export * from './categories'
export * from './entries'
export * from './environments'
export * from './selection'
export * from './settings'
