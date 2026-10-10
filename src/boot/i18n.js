// Initialise the UI language before the app renders, so there is no flash of
// the wrong language. The persisted setting lives in the Rust settings store
// (`Settings.language`); fall back to auto-detection when it is unavailable.
import { boot } from 'quasar/wrappers'
import { setLocale } from '@/i18n'
import { backend } from '@/services/backend'

export default boot(async () => {
  let setting = 'auto'
  try {
    const s = await backend.loadSettings()
    if (s && typeof s.language === 'string') setting = s.language
  } catch (e) {
    console.warn('load language setting failed', e)
  }
  setLocale(setting)
})
