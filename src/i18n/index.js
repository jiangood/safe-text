// Tiny dependency-free i18n layer.
//
// Instead of pulling in vue-i18n, the app keeps a reactive `locale` ref and a
// plain message lookup. Using `t()` inside a template (or a `computed`) tracks
// `locale`, so switching the language re-renders automatically.
//
// Supported settings (`Settings.language` on the Rust side):
//   'auto' → follow the system/browser language (default)
//   'zh'   → Simplified Chinese
//   'en'   → English

import { ref } from 'vue'
import { Lang } from 'quasar'
import quasarEnUS from 'quasar/lang/en-US'
import quasarZhCN from 'quasar/lang/zh-CN'

import en from './en-US'
import zh from './zh-CN'

const MESSAGES = {
  'en-US': en,
  'zh-CN': zh
}

const QUASAR_LANGS = {
  'en-US': quasarEnUS,
  'zh-CN': quasarZhCN
}

export const FALLBACK_LOCALE = 'en-US'

/** Currently active locale, e.g. 'zh-CN'. Reactive. */
export const locale = ref(FALLBACK_LOCALE)

/** Language choices shown in Settings. `labelKey` is resolved with `t()`. */
export const languageOptions = [
  { value: 'auto', labelKey: 'settings.languageAuto' },
  { value: 'zh', labelKey: 'settings.languageZh' },
  { value: 'en', labelKey: 'settings.languageEn' }
]

/** Clamp an arbitrary persisted value to a known setting. */
export function normalizeLanguageSetting (setting) {
  return setting === 'zh' || setting === 'en' || setting === 'auto' ? setting : 'auto'
}

/** Best-effort locale from the host: any `zh*` → Simplified Chinese. */
export function detectLocale () {
  const nav = (navigator.languages && navigator.languages[0]) || navigator.language || ''
  return String(nav).toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}

/** Resolve a persisted setting ('auto' | 'zh' | 'en') to a concrete locale. */
export function resolveLocale (setting) {
  const s = normalizeLanguageSetting(setting)
  if (s === 'zh') return 'zh-CN'
  if (s === 'en') return 'en-US'
  return detectLocale()
}

/**
 * Apply a language setting: update the reactive locale, the `<html lang>`
 * attribute and the Quasar component language pack. Returns the resolved
 * locale.
 */
export function setLocale (setting) {
  const resolved = resolveLocale(setting)
  locale.value = resolved
  const pack = QUASAR_LANGS[resolved]
  if (pack) Lang.set(pack)
  return resolved
}

function lookup (obj, key) {
  return key.split('.').reduce((acc, part) => (acc == null ? acc : acc[part]), obj)
}

/**
 * Translate `key` for the active locale. Falls back to English, then to the
 * key itself. `{name}` placeholders are filled from `params`.
 */
export function t (key, params) {
  let value = lookup(MESSAGES[locale.value], key)
  if (value == null) value = lookup(MESSAGES[FALLBACK_LOCALE], key)
  if (value == null) return key

  if (params) {
    value = String(value).replace(/\{(\w+)\}/g, (match, name) =>
      params[name] == null ? match : String(params[name])
    )
  }
  return value
}
