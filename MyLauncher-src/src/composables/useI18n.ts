// ═══════════════════════════════════════════
// useI18n —— i18n 运行时的组合式封装
//
// 组件内用法：
//   const { t } = useI18n()
//   <button>{{ t('common.save') }}</button>
// 模板中也可用全局 $t（main.ts 挂载到 globalProperties）。
// ═══════════════════════════════════════════

import { t, locale, setLocale } from '@/i18n'

export function useI18n() {
  return { t, locale, setLocale }
}
