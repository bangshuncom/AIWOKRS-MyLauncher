// ════════════════════════════════════════════
// 多语言（i18n）运行时
//
// 语言文件由后端释放到 <exe>/lang/*.json，经 read_language 命令读取。
// 本模块只持有内存状态：
//   locale   —— 当前语言代码（响应式，切换时全界面重渲染）
//   messages —— 当前语言消息表（整体替换，不做增量合并）
//
// 翻译函数 t(key, params) 带三级回退链：
//   ① 当前语言 messages → ② 内置 zh-CN（编译期内置 JSON）→ ③ key 原文
// 任一级未命中即向右回退；三级均未命中时返回 key 本身并在开发模式打印警告。
// 内置 zh-CN 与后端 include_str! 同源（src-tauri/lang/zh-CN.json），
// 因此前端离线兜底与后端释放模板永远是同一份内容。
// ════════════════════════════════════════════

import { ref } from 'vue'
import { read_language } from '@/api'
import zhCN from '../../src-tauri/lang/zh-CN.json'

export type Messages = Record<string, string>

/** 内置回落语言（权威 key 源） */
export const FALLBACK_LOCALE = 'zh-CN'

export const locale = ref(FALLBACK_LOCALE)
const messages = ref<Messages>({})
/** 内置兜底消息表（编译期内置，进程生命周期内不变） */
const fallbackMessages = ref<Messages>(zhCN.messages as Messages)

/** 占位符插值：{name} → params.name（未提供的占位符原样保留） */
function interpolate(text: string, params?: Record<string, string | number>): string {
  if (!params) return text
  return text.replace(/\{(\w+)\}/g, (m, key) =>
    Object.prototype.hasOwnProperty.call(params, key) ? String(params[key]) : m
  )
}

/**
 * 翻译函数（三级回退链）。
 * 用法：t('common.save') / t('main.entryDeleted', { name: entry.name })
 */
export function t(key: string, params?: Record<string, string | number>): string {
  let text = messages.value[key]
  if (text === undefined) text = fallbackMessages.value[key]
  if (text === undefined) {
    if (import.meta.env.DEV) {
      console.warn(`[i18n] 缺失的翻译 key: ${key}`)
    }
    return key
  }
  return interpolate(text, params)
}

/**
 * 切换语言：读取语言文件并整体替换消息表。
 * 文件不存在/损坏时返回 false（messages 保持原值，界面不空白）。
 */
export async function setLocale(code: string): Promise<boolean> {
  try {
    const file = await read_language(code)
    messages.value = file.messages
    locale.value = file.code
    return true
  } catch (e) {
    console.error(`[i18n] 加载语言失败: ${code}`, e)
    return false
  }
}

/**
 * 启动时初始化语言。
 * savedCode 为空（老配置无 language 字段）时跟随系统语言，未识别回落 zh-CN；
 * 加载失败同样回落 zh-CN，保证界面始终有完整文案。
 */
export async function initI18n(savedCode: string): Promise<void> {
  let code = savedCode
  if (!code) {
    const sys = (navigator.language || '').replace('_', '-')
    code = ['zh-CN', 'zh-TW', 'en-US'].includes(sys) ? sys : FALLBACK_LOCALE
  }
  const ok = await setLocale(code)
  if (!ok && code !== FALLBACK_LOCALE) {
    await setLocale(FALLBACK_LOCALE)
  }
}
