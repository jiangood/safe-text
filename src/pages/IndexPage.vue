<template>
  <q-layout view="hHh lpR fFf">
    <q-header elevated class="bg-primary text-white">
      <q-toolbar>
        <q-toolbar-title class="row items-center no-wrap">
          <q-icon name="lock" size="20px" class="q-mr-sm" />
          <span class="text-weight-medium">SafeText</span>
        </q-toolbar-title>

        <q-btn
          flat dense no-caps icon="note_add" :label="t('toolbar.new')"
          :disable="busy" @click="onNew"
        />
        <q-btn
          flat dense no-caps icon="folder_open" :label="t('toolbar.open')"
          :disable="busy" @click="onOpen"
        />
        <q-btn
          flat dense no-caps icon="history" :label="t('toolbar.recent')"
          :disable="busy"
        >
          <q-menu auto-close>
            <q-list style="min-width: 280px; max-width: 420px">
              <template v-if="recentFiles.length">
                <q-item
                  v-for="p in recentFiles"
                  :key="p"
                  clickable v-close-popup
                  @click="onOpenRecent(p)"
                >
                  <q-item-section avatar>
                    <q-icon name="description" />
                  </q-item-section>
                  <q-item-section>
                    <q-item-label>{{ baseName(p) }}</q-item-label>
                    <q-item-label caption lines="1">{{ p }}</q-item-label>
                  </q-item-section>
                </q-item>
                <q-separator />
                <q-item clickable v-close-popup @click="onClearRecent">
                  <q-item-section avatar>
                    <q-icon name="delete_sweep" />
                  </q-item-section>
                  <q-item-section>{{ t('recent.clear') }}</q-item-section>
                </q-item>
              </template>
              <q-item v-else>
                <q-item-section avatar>
                  <q-icon name="info" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>{{ t('recent.empty') }}</q-item-label>
                  <q-item-label caption>
                    {{ rememberRecent
                      ? t('recent.emptyHintEnabled')
                      : t('recent.emptyHintDisabled') }}
                  </q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-menu>
        </q-btn>
        <q-separator dark vertical inset class="q-mx-xs" />
        <q-btn
          flat dense no-caps icon="save" :label="t('toolbar.save')"
          :disable="busy || !unlocked" @click="onSave"
        />
        <q-btn
          flat dense no-caps icon="save_as" :label="t('toolbar.saveAs')"
          :disable="busy || !unlocked" @click="onSaveAs"
        />
        <q-separator dark vertical inset class="q-mx-xs" />
        <q-btn
          flat dense no-caps icon="key" :label="t('toolbar.changePassword')"
          :disable="busy || !unlocked" @click="onChangePassword"
        />
        <q-btn
          flat dense no-caps icon="lock_outline" :label="t('toolbar.lock')"
          :disable="busy || !unlocked" @click="onLock(false)"
        />
        <q-space />
        <q-btn flat dense round icon="settings" :disable="busy" @click="openSettings">
          <q-tooltip>{{ t('toolbar.settings') }}</q-tooltip>
        </q-btn>
      </q-toolbar>
    </q-header>

    <q-page-container>
      <q-page class="column no-wrap editor-page">
        <textarea
          ref="textareaRef"
          v-model="text"
          class="editor"
          spellcheck="false"
          autocomplete="off"
          autocapitalize="off"
          :placeholder="placeholder"
          @input="onEdit"
        ></textarea>

        <div class="status-bar row items-center q-px-sm">
          <q-icon
            :name="unlocked ? 'lock' : 'lock_open'"
            :color="unlocked ? 'positive' : 'grey'"
            size="16px"
            class="q-mr-xs"
          />
          <span class="text-caption">{{ statusText }}</span>
          <q-space />
          <span class="text-caption text-grey-6 q-mr-md">
            AES-256-GCM · Argon2id
          </span>
          <span class="text-caption" :class="dirty ? 'text-orange' : 'text-grey-6'">
            {{ dirty ? t('status.unsaved') : t('status.saved') }}
          </span>
        </div>
      </q-page>
    </q-page-container>

    <!-- Password prompt -->
    <q-dialog v-model="pw.open" persistent @keyup.enter="pwSubmit">
      <q-card style="min-width: 340px">
        <q-card-section class="row items-center">
          <q-icon name="lock" class="q-mr-sm" />
          <div class="text-h6">{{ pw.title }}</div>
        </q-card-section>

        <q-card-section class="q-gutter-md">
          <q-input
            v-model="pw.value"
            :type="pw.reveal ? 'text' : 'password'"
            :label="pw.mode === 'open' ? t('password.label') : t('password.newLabel')"
            filled autofocus
          >
            <template #append>
              <q-icon
                :name="pw.reveal ? 'visibility_off' : 'visibility'"
                class="cursor-pointer"
                @click="pw.reveal = !pw.reveal"
              />
            </template>
          </q-input>

          <q-input
            v-if="pw.mode !== 'open'"
            v-model="pw.confirm"
            :type="pw.reveal ? 'text' : 'password'"
            :label="t('password.confirmLabel')"
            filled
          />

          <div v-if="pw.error" class="text-negative text-caption">
            {{ pw.error }}
          </div>
          <div
            v-if="pw.mode === 'new'"
            class="text-caption text-grey-6"
          >
            {{ t('password.warnNotSaved') }}
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat no-caps :label="t('common.cancel')" @click="pwCancel" />
          <q-btn unelevated no-caps color="primary" :label="t('common.ok')" @click="pwSubmit" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- Settings -->
    <q-dialog v-model="settings.open">
      <q-card style="min-width: 380px">
        <q-card-section class="row items-center">
          <q-icon name="settings" class="q-mr-sm" />
          <div class="text-h6">{{ t('settings.title') }}</div>
        </q-card-section>

        <q-card-section class="q-gutter-md">
          <q-select
            v-model="settings.language"
            :options="languageSelectOptions"
            :label="t('settings.language')"
            emit-value
            map-options
            options-dense
            filled
          />

          <q-input
            v-model.number="settings.autolock"
            type="number"
            :label="t('settings.autolock')"
            min="0"
            filled
          />

          <q-toggle
            v-model="settings.rememberRecent"
            :label="t('settings.rememberRecent')"
            left-label
          />
          <div class="text-caption text-grey-6">
            {{ t('settings.rememberRecentHint') }}
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat no-caps :label="t('common.cancel')" @click="settings.open = false" />
          <q-btn unelevated no-caps color="primary" :label="t('common.save')" @click="onSaveSettings" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- Busy overlay -->
    <q-inner-loading :showing="busy">
      <q-spinner-gears size="42px" color="primary" />
      <div class="q-mt-sm text-grey-7">{{ t('common.processing') }}</div>
    </q-inner-loading>
  </q-layout>
</template>

<script setup>
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { Dialog, Notify } from 'quasar'
import { backend, parseError } from '@/services/backend'
import {
  t,
  locale,
  setLocale,
  normalizeLanguageSetting,
  languageOptions
} from '@/i18n'

const text = ref('')
const filePath = ref(null)
const fileName = ref(t('common.untitled'))
const dirty = ref(false)
const unlocked = ref(false)
const busy = ref(false)
const textareaRef = ref(null)

// `pathScoped` tracks whether the current file's path carries a dialog-granted
// fs scope. Files opened from the recent list / drag & drop do not, so they are
// read & written through the Rust commands instead of the fs plugin.
const pathScoped = ref(false)

const recentFiles = ref([])
const rememberRecent = ref(false)

const autolockMinutes = ref(5)
let lastActivity = Date.now()
let idleTimer = null
let unlistenDrop = null

const settings = reactive({
  open: false,
  autolock: 5,
  rememberRecent: false,
  language: 'auto'
})

const languageSelectOptions = computed(() =>
  languageOptions.map((o) => ({ label: t(o.labelKey), value: o.value }))
)

const pw = reactive({
  open: false,
  title: '',
  mode: 'open',
  value: '',
  confirm: '',
  error: '',
  reveal: false,
  resolver: null
})

const placeholder = computed(() =>
  unlocked.value
    ? t('editor.placeholderUnlocked')
    : t('editor.placeholderLocked')
)

const statusText = computed(() => {
  if (!unlocked.value) return t('status.noDoc')
  return `${fileName.value}${filePath.value ? '' : t('status.notOnDisk')}`
})

// ---------------------------------------------------------------------------
// Notifications / confirmations
// ---------------------------------------------------------------------------
function showError (code, detail) {
  let message
  if (code === 'unknown') {
    // Plugin (dialog / fs) errors have no stable code – show the raw text.
    message = detail || t('errors.unknown')
  } else {
    const key = `errors.${code}`
    const translated = t(key, { detail })
    message = translated === key
      ? (detail || t('errors.unknown'))
      : translated
  }
  Dialog({
    title: t('common.errorTitle'),
    message,
    ok: { label: t('common.gotIt'), color: 'negative', unelevated: true, noCaps: true },
    persistent: true
  })
}

function notifyError (e) {
  const { code, detail } = parseError(e)
  showError(code, detail)
}

function notifyOk (msg) {
  Notify.create({
    type: 'positive',
    message: msg,
    position: 'bottom',
    timeout: 1500
  })
}

function confirmDialog (message) {
  return new Promise((resolve) => {
    Dialog({
      title: t('common.confirmTitle'),
      message,
      cancel: { label: t('common.cancel'), noCaps: true, flat: true },
      ok: { label: t('common.ok'), color: 'primary', unelevated: true, noCaps: true },
      persistent: true
    })
      .onOk(() => resolve(true))
      .onCancel(() => resolve(false))
      .onDismiss(() => resolve(false))
  })
}

function confirmDiscard (actionKey) {
  if (!dirty.value) return Promise.resolve(true)
  return confirmDialog(t('dialog.discard', { action: t(actionKey) }))
}

// ---------------------------------------------------------------------------
// Password prompt
// ---------------------------------------------------------------------------
function promptPassword (title, mode) {
  return new Promise((resolve) => {
    Object.assign(pw, {
      open: true,
      title,
      mode,
      value: '',
      confirm: '',
      error: '',
      reveal: false,
      resolver: resolve
    })
  })
}

function pwSubmit () {
  if (!pw.value) {
    pw.error = t('password.errEmpty')
    return
  }
  if (pw.mode !== 'open' && pw.value !== pw.confirm) {
    pw.error = t('password.errMismatch')
    return
  }
  if (pw.mode !== 'open' && pw.value.length < 6) {
    pw.error = t('password.errTooShort')
    return
  }
  const resolve = pw.resolver
  pw.open = false
  pw.resolver = null
  if (resolve) resolve(pw.value)
}

function pwCancel () {
  const resolve = pw.resolver
  pw.open = false
  pw.resolver = null
  if (resolve) resolve(null)
}

// ---------------------------------------------------------------------------
// File helpers
// ---------------------------------------------------------------------------
function baseName (path) {
  if (!path) return t('common.untitled')
  const clean = String(path).replace(/[\\/]+$/, '')
  const parts = clean.split(/[\\/]/)
  let name = parts[parts.length - 1] || t('common.untitled')
  // Strip a trailing query/fragment that content:// URIs may carry.
  const q = name.indexOf('?')
  if (q >= 0) name = name.slice(0, q)
  return name
}

function setOpened (path) {
  filePath.value = path
  fileName.value = baseName(path)
  dirty.value = false
  unlocked.value = true
  updateTitle()
}

function resetDoc () {
  text.value = ''
  filePath.value = null
  fileName.value = t('common.untitled')
  dirty.value = false
  unlocked.value = false
  pathScoped.value = false
  updateTitle()
}

function updateTitle () {
  const lock = unlocked.value ? t('title.encrypted') : t('title.unlocked')
  const star = dirty.value ? '*' : ''
  document.title = `${star}${fileName.value} - SafeText [${lock}]`
}

function onEdit () {
  dirty.value = true
  resetIdle()
  updateTitle()
}

// Write the ciphertext back to the current path, choosing the fs plugin for
// dialog-scoped paths and the Rust command otherwise (drag & drop / recent).
async function writeCurrentFile (bytes) {
  if (!filePath.value) return
  if (pathScoped.value) {
    await backend.writeFile(filePath.value, bytes)
  } else {
    await backend.writeFileByPath(filePath.value, bytes)
  }
}

// Remember a path in the opt-in recent list. content:// URIs are skipped
// because they are not reopenable by path.
async function rememberRecentFile (path) {
  if (!rememberRecent.value || !path || String(path).includes('://')) return
  try {
    const s = await backend.addRecentFile(path)
    if (s && Array.isArray(s.recent_files)) recentFiles.value = s.recent_files
  } catch (e) {
    console.warn('record recent file failed', e)
  }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------
async function onNew () {
  if (!(await confirmDiscard('actions.new'))) return
  const password = await promptPassword(t('password.titleNew'), 'new')
  if (password === null) return

  busy.value = true
  try {
    text.value = ''
    const bytes = await backend.newDocument(password, text.value)
    const path = await backend.pickSavePath(t('common.untitled'))
    if (path) {
      await backend.writeFile(path, bytes)
      setOpened(path)
      pathScoped.value = true
      notifyOk(t('notify.created'))
    } else {
      // Created in memory only; key is already cached in Rust.
      filePath.value = null
      fileName.value = t('common.untitled')
      unlocked.value = true
      dirty.value = true
      updateTitle()
    }
  } catch (e) {
    notifyError(e)
  } finally {
    busy.value = false
  }
}

async function onOpen () {
  if (!(await confirmDiscard('actions.open'))) return
  let path
  try {
    path = await backend.pickOpenPath()
  } catch (e) {
    notifyError(e)
    return
  }
  if (!path) return
  await openFromPath(path, () => backend.readFile(path), true)
}

async function onOpenRecent (path) {
  if (!(await confirmDiscard('actions.open'))) return
  await openFromPath(path, () => backend.readFileByPath(path), false)
}

async function openFromPath (path, reader, viaDialog = false) {
  busy.value = true
  let bytes
  try {
    bytes = await reader()
  } catch (e) {
    busy.value = false
    notifyError(e)
    return
  }
  busy.value = false

  if (!(await backend.probeFile(bytes))) {
    showError('not_encrypted')
    return
  }

  // Retry loop so the user can re-enter the password after a failure.
  for (;;) {
    const password = await promptPassword(t('password.titleOpen'), 'open')
    if (password === null) return
    busy.value = true
    try {
      const plain = await backend.openDocument(password, bytes)
      text.value = plain
      setOpened(path)
      pathScoped.value = viaDialog
      await rememberRecentFile(path)
      notifyOk(t('notify.opened'))
      return
    } catch (e) {
      busy.value = false
      const { code, detail } = parseError(e)
      if (code === 'bad_password') {
        await new Promise((resolve) => {
          Dialog({
            title: t('dialog.cannotOpen'),
            message: t('errors.bad_password'),
            ok: { label: t('common.retry'), color: 'primary', unelevated: true, noCaps: true }
          }).onOk(resolve).onDismiss(resolve)
        })
        continue
      }
      showError(code, detail)
      return
    } finally {
      busy.value = false
    }
  }
}

async function requireUnlocked () {
  if (unlocked.value) return true
  showError('no_session')
  return false
}

async function onSave () {
  if (!(await requireUnlocked())) return
  if (!filePath.value) return onSaveAs()

  busy.value = true
  try {
    const bytes = await backend.saveDocument(text.value)
    await writeCurrentFile(bytes)
    dirty.value = false
    updateTitle()
    notifyOk(t('notify.saved'))
  } catch (e) {
    notifyError(e)
  } finally {
    busy.value = false
  }
}

async function onSaveAs () {
  if (!(await requireUnlocked())) return
  busy.value = true
  try {
    const bytes = await backend.saveDocument(text.value)
    const path = await backend.pickSavePath(fileName.value || t('common.untitled'))
    if (!path) return
    await backend.writeFile(path, bytes)
    setOpened(path)
    pathScoped.value = true
    notifyOk(t('notify.saved'))
  } catch (e) {
    notifyError(e)
  } finally {
    busy.value = false
  }
}

async function onChangePassword () {
  if (!(await requireUnlocked())) return
  const password = await promptPassword(t('password.titleChange'), 'change')
  if (password === null) return

  busy.value = true
  try {
    const bytes = await backend.changePassword(password, text.value)
    if (filePath.value) {
      await writeCurrentFile(bytes)
      dirty.value = false
      updateTitle()
      notifyOk(t('notify.passwordChangedSaved'))
    } else {
      const path = await backend.pickSavePath(fileName.value || t('common.untitled'))
      if (path) {
        await backend.writeFile(path, bytes)
        setOpened(path)
        pathScoped.value = true
      } else {
        dirty.value = true
      }
      notifyOk(t('notify.passwordChanged'))
    }
  } catch (e) {
    notifyError(e)
  } finally {
    busy.value = false
  }
}

async function onLock (force) {
  if (!unlocked.value && !text.value) return
  if (!force && !(await confirmDiscard('actions.lock'))) return

  try {
    await backend.lock()
  } catch (e) {
    // Even if the backend call fails, drop local state.
    console.error(e)
  }
  resetDoc()
  if (!force) notifyOk(t('notify.locked'))
}

// ---------------------------------------------------------------------------
// Auto-lock
// ---------------------------------------------------------------------------
function resetIdle () {
  lastActivity = Date.now()
}

function startIdleWatcher () {
  clearInterval(idleTimer)
  idleTimer = setInterval(() => {
    const minutes = autolockMinutes.value
    if (!unlocked.value || !minutes || minutes <= 0) return
    if (Date.now() - lastActivity >= minutes * 60 * 1000) {
      onLock(true)
    }
  }, 5000)
}

async function loadSettings () {
  try {
    const s = await backend.loadSettings()
    if (s && typeof s.autolock_minutes === 'number') {
      autolockMinutes.value = s.autolock_minutes
    }
    rememberRecent.value = !!(s && s.remember_recent)
    recentFiles.value = Array.isArray(s?.recent_files) ? s.recent_files : []

    // Keep the UI in sync with the persisted language (the boot file already
    // applied it; this covers the case where the boot ran before the store
    // was ready).
    const lang = normalizeLanguageSetting(s?.language)
    settings.language = lang
    setLocale(lang)
  } catch (e) {
    console.warn('load settings failed', e)
  }
}

function openSettings () {
  settings.autolock = autolockMinutes.value
  settings.rememberRecent = rememberRecent.value
  settings.open = true
}

async function onSaveSettings () {
  const minutes = Number(settings.autolock)
  autolockMinutes.value = Number.isFinite(minutes) && minutes >= 0
    ? Math.floor(minutes)
    : 5
  rememberRecent.value = !!settings.rememberRecent
  // The backend also wipes paths when remembering is off; keep the UI in sync.
  if (!rememberRecent.value) recentFiles.value = []

  const lang = normalizeLanguageSetting(settings.language)

  busy.value = true
  try {
    await backend.saveSettings({
      autolock_minutes: autolockMinutes.value,
      remember_recent: rememberRecent.value,
      recent_files: recentFiles.value,
      language: lang
    })
    // Apply the language only after it has been persisted successfully.
    setLocale(lang)
    resetIdle()
    settings.open = false
    notifyOk(t('settings.saved'))
  } catch (e) {
    notifyError(e)
  } finally {
    busy.value = false
  }
}

async function onClearRecent () {
  if (recentFiles.value.length && !(await confirmDialog(t('recent.clearConfirm')))) {
    return
  }
  try {
    const s = await backend.clearRecentFiles()
    recentFiles.value = Array.isArray(s?.recent_files) ? s.recent_files : []
    notifyOk(t('recent.cleared'))
  } catch (e) {
    notifyError(e)
  }
}

// ---------------------------------------------------------------------------
// Keyboard shortcuts
// ---------------------------------------------------------------------------
function onKey (ev) {
  if (!(ev.ctrlKey || ev.metaKey)) return
  const key = ev.key.toLowerCase()
  if (key === 'n') { ev.preventDefault(); onNew() }
  else if (key === 'o') { ev.preventDefault(); onOpen() }
  else if (key === 's' && ev.shiftKey) { ev.preventDefault(); onSaveAs() }
  else if (key === 's') { ev.preventDefault(); onSave() }
  else if (key === 'l') { ev.preventDefault(); onLock(false) }
}

// Keep the (localized) window title in sync when the language changes.
watch(locale, updateTitle)

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------
onMounted(async () => {
  updateTitle()
  await loadSettings()
  startIdleWatcher()

  window.addEventListener('keydown', onKey)
  window.addEventListener('mousemove', resetIdle)
  window.addEventListener('keydown', resetIdle)
  window.addEventListener('focus', resetIdle)

  try {
    unlistenDrop = await backend.onDragDrop(async (event) => {
      const payload = event?.payload
      if (!payload || payload.type !== 'drop') return
      const paths = payload.paths || []
      if (paths.length === 0) return
      if (!(await confirmDiscard('actions.open'))) return
      const path = paths[0]
      await openFromPath(path, () => backend.readFileByPath(path))
    })
  } catch (e) {
    console.warn('drag-drop listener failed', e)
  }

  textareaRef.value?.focus()
})

onUnmounted(() => {
  clearInterval(idleTimer)
  if (unlistenDrop) unlistenDrop()
  window.removeEventListener('keydown', onKey)
  window.removeEventListener('mousemove', resetIdle)
  window.removeEventListener('keydown', resetIdle)
  window.removeEventListener('focus', resetIdle)
})
</script>

<style scoped>
.editor-page {
  height: 100%;
  background: var(--q-dark-page, #f5f5f5);
}

.editor {
  flex: 1 1 auto;
  width: 100%;
  border: none;
  outline: none;
  resize: none;
  padding: 12px 14px;
  font-family: 'Cascadia Mono', 'Consolas', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  color: #1b1b1b;
  background: #ffffff;
  tab-size: 2;
}

.body--dark .editor {
  color: #e0e0e0;
  background: #121212;
}

.status-bar {
  flex: 0 0 auto;
  height: 30px;
  background: #ececec;
  border-top: 1px solid #d6d6d6;
}

.body--dark .status-bar {
  background: #1e1e1e;
  border-top-color: #333;
}
</style>
