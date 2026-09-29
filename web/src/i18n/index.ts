import { readonly, ref } from 'vue'
import { zh } from './zh'

export type Language = 'en' | 'zh'
const STORAGE_KEY = 'language'

function readLanguage(): Language {
  try {
    return localStorage.getItem(STORAGE_KEY) === 'zh' ? 'zh' : 'en'
  } catch {
    return 'en'
  }
}

const currentLanguage = ref<Language>(readLanguage())
export const language = readonly(currentLanguage)

export function setLanguage(value: Language) {
  if (value !== 'en' && value !== 'zh') return
  currentLanguage.value = value
  document.documentElement.lang = value === 'zh' ? 'zh-CN' : 'en'
  try {
    localStorage.setItem(STORAGE_KEY, value)
  } catch {
    // Language switching still works when browser storage is unavailable.
  }
}

/** Translate UI messages only; library metadata and server responses stay intact. */
export function tr(message: string, params: Record<string, string | number> = {}): string {
  const template = currentLanguage.value === 'zh' && Object.prototype.hasOwnProperty.call(zh, message)
    ? zh[message as keyof typeof zh]
    : message
  return template.replace(/\{(\w+)\}/g, (placeholder, key: string) =>
    Object.prototype.hasOwnProperty.call(params, key) ? String(params[key]) : placeholder,
  )
}
