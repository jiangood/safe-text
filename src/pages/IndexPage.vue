<template>
  <q-layout view="hHh lpR fFf">
    <q-header elevated class="bg-primary text-white">
      <q-toolbar>
        <q-toolbar-title class="row items-center no-wrap">
          <q-icon name="lock" size="20px" class="q-mr-sm" />
          <span class="text-weight-medium">SafeText</span>
        </q-toolbar-title>

        <q-btn
          flat dense no-caps icon="note_add" label="新建"
          :disable="busy" @click="onNew"
        />
        <q-btn
          flat dense no-caps icon="folder_open" label="打开"
          :disable="busy" @click="onOpen"
        />
        <q-btn
          flat dense no-caps icon="history" label="最近"
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
                  <q-item-section>清空最近记录</q-item-section>
                </q-item>
              </template>
              <q-item v-else>
                <q-item-section avatar>
                  <q-icon name="info" />
                </q-item-section>
                <q-item-section>
                  <q-item-label>暂无最近文件</q-item-label>
                  <q-item-label caption>
                    {{ rememberRecent
                      ? '打开过的文件会出现在这里'
                      : '在设置中启用“记住最近文件”后开始记录' }}
                  </q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </q-menu>
        </q-btn>
        <q-separator dark vertical inset class="q-mx-xs" />
        <q-btn
          flat dense no-caps icon="save" label="保存"
          :disable="busy || !unlocked" @click="onSave"
        />
        <q-btn
          flat dense no-caps icon="save_as" label="另存为"
          :disable="busy || !unlocked" @click="onSaveAs"
        />
        <q-separator dark vertical inset class="q-mx-xs" />
        <q-btn
          flat dense no-caps icon="key" label="改密码"
          :disable="busy || !unlocked" @click="onChangePassword"
        />
        <q-btn
          flat dense no-caps icon="lock_outline" label="锁定"
          :disable="busy || !unlocked" @click="onLock(false)"
        />
        <q-space />
        <q-btn flat dense round icon="settings" :disable="busy" @click="openSettings">
          <q-tooltip>设置</q-tooltip>
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
            {{ dirty ? '未保存' : '已保存' }}
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
            :label="pw.mode === 'open' ? '密码' : '新密码'"
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
            label="确认密码"
            filled
          />

          <div v-if="pw.error" class="text-negative text-caption">
            {{ pw.error }}
          </div>
          <div
            v-if="pw.mode === 'new'"
            class="text-caption text-grey-6"
          >
            密码不会被保存，请务必牢记；遗失后无法恢复文件内容。
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat no-caps label="取消" @click="pwCancel" />
          <q-btn unelevated no-caps color="primary" label="确定" @click="pwSubmit" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- Settings -->
    <q-dialog v-model="settings.open">
      <q-card style="min-width: 380px">
        <q-card-section class="row items-center">
          <q-icon name="settings" class="q-mr-sm" />
          <div class="text-h6">设置</div>
        </q-card-section>

        <q-card-section class="q-gutter-md">
          <q-input
            v-model.number="settings.autolock"
            type="number"
            label="闲置自动锁定（分钟，0 表示关闭）"
            min="0"
            filled
          />

          <q-toggle
            v-model="settings.rememberRecent"
            label="记住最近打开的文件"
            left-label
          />
          <div class="text-caption text-grey-6">
            启用后仅在本地设置文件中保存文件路径，方便下次快速打开；关闭并保存将立即清除全部记录。
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat no-caps label="取消" @click="settings.open = false" />
          <q-btn unelevated no-caps color="primary" label="保存" @click="onSaveSettings" />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <!-- Busy overlay -->
    <q-inner-loading :showing="busy">
      <q-spinner-gears size="42px" color="primary" />
      <div class="q-mt-sm text-grey-7">处理中…</div>
    </q-inner-loading>
  </q-layout>
</template>

<script setup>
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { Dialog, Notify } from 'quasar'
import { backend } from '@/services/backend'

const text = ref('')
const filePath = ref(null)
const fileName = ref('未命名.txt')
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
  rememberRecent: false
})

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
    ? '在此输入内容，保存时将自动加密…'
    : '点击“新建”创建加密文档，或“打开”已有加密 .txt 文件'
)

const statusText = computed(() => {
  if (!unlocked.value) return '未打开文档'
  return `${fileName.value}${filePath.value ? '' : '（尚未保存到磁盘）'}`
})

// ---------------------------------------------------------------------------
// Notifications / confirmations
// ---------------------------------------------------------------------------
function notifyError (e) {
  const msg = typeof e === 'string' ? e : (e?.message || String(e))
  Dialog({
    title: '出错了',
    message: msg,
    ok: { label: '知道了', color: 'negative', unelevated: true, noCaps: true },
    persistent: true
  })
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
      title: '确认',
      message,
      cancel: { label: '取消', noCaps: true, flat: true },
      ok: { label: '确定', color: 'primary', unelevated: true, noCaps: true },
      persistent: true
    })
      .onOk(() => resolve(true))
      .onCancel(() => resolve(false))
      .onDismiss(() => resolve(false))
  })
}

function confirmDiscard (action) {
  if (!dirty.value) return Promise.resolve(true)
  return confirmDialog(`有未保存的修改，继续${action}将放弃这些修改。是否继续？`)
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
    pw.error = '请输入密码'
    return
  }
  if (pw.mode !== 'open' && pw.value !== pw.confirm) {
    pw.error = '两次输入的密码不一致'
    return
  }
  if (pw.mode !== 'open' && pw.value.length < 6) {
    pw.error = '密码至少需要 6 个字符'
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
  if (!path) return '未命名.txt'
  const clean = String(path).replace(/[\\/]+$/, '')
  const parts = clean.split(/[\\/]/)
  let name = parts[parts.length - 1] || '未命名.txt'
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
  fileName.value = '未命名.txt'
  dirty.value = false
  unlocked.value = false
  pathScoped.value = false
  updateTitle()
}

function updateTitle () {
  const lock = unlocked.value ? '已加密' : '未锁定'
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
  if (!(await confirmDiscard('新建'))) return
  const password = await promptPassword('为新文档设置密码', 'new')
  if (password === null) return

  busy.value = true
  try {
    text.value = ''
    const bytes = await backend.newDocument(password, text.value)
    const path = await backend.pickSavePath('未命名.txt')
    if (path) {
      await backend.writeFile(path, bytes)
      setOpened(path)
      pathScoped.value = true
      notifyOk('已创建并保存')
    } else {
      // Created in memory only; key is already cached in Rust.
      filePath.value = null
      fileName.value = '未命名.txt'
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
  if (!(await confirmDiscard('打开'))) return
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
  if (!(await confirmDiscard('打开'))) return
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
    notifyError('不是有效加密文件')
    return
  }

  // Retry loop so the user can re-enter the password after a failure.
  for (;;) {
    const password = await promptPassword('输入密码以打开', 'open')
    if (password === null) return
    busy.value = true
    try {
      const plain = await backend.openDocument(password, bytes)
      text.value = plain
      setOpened(path)
      pathScoped.value = viaDialog
      await rememberRecentFile(path)
      notifyOk('已打开')
      return
    } catch (e) {
      busy.value = false
      const msg = String(e)
      if (msg.includes('密码错误')) {
        await new Promise((resolve) => {
          Dialog({
            title: '无法打开',
            message: msg,
            ok: { label: '重试', color: 'primary', unelevated: true, noCaps: true }
          }).onOk(resolve).onDismiss(resolve)
        })
        continue
      }
      notifyError(msg)
      return
    } finally {
      busy.value = false
    }
  }
}

async function requireUnlocked () {
  if (unlocked.value) return true
  notifyError('没有已解锁的文档')
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
    notifyOk('已保存')
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
    const path = await backend.pickSavePath(fileName.value || '未命名.txt')
    if (!path) return
    await backend.writeFile(path, bytes)
    setOpened(path)
    pathScoped.value = true
    notifyOk('已保存')
  } catch (e) {
    notifyError(e)
  } finally {
    busy.value = false
  }
}

async function onChangePassword () {
  if (!(await requireUnlocked())) return
  const password = await promptPassword('设置新密码', 'change')
  if (password === null) return

  busy.value = true
  try {
    const bytes = await backend.changePassword(password, text.value)
    if (filePath.value) {
      await writeCurrentFile(bytes)
      dirty.value = false
      updateTitle()
      notifyOk('密码已修改并保存')
    } else {
      const path = await backend.pickSavePath(fileName.value || '未命名.txt')
      if (path) {
        await backend.writeFile(path, bytes)
        setOpened(path)
        pathScoped.value = true
      } else {
        dirty.value = true
      }
      notifyOk('密码已修改')
    }
  } catch (e) {
    notifyError(e)
  } finally {
    busy.value = false
  }
}

async function onLock (force) {
  if (!unlocked.value && !text.value) return
  if (!force && !(await confirmDiscard('锁定'))) return

  try {
    await backend.lock()
  } catch (e) {
    // Even if the backend call fails, drop local state.
    console.error(e)
  }
  resetDoc()
  if (!force) notifyOk('已锁定')
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

  busy.value = true
  try {
    await backend.saveSettings({
      autolock_minutes: autolockMinutes.value,
      remember_recent: rememberRecent.value,
      recent_files: recentFiles.value
    })
    resetIdle()
    settings.open = false
    notifyOk('设置已保存')
  } catch (e) {
    notifyError(e)
  } finally {
    busy.value = false
  }
}

async function onClearRecent () {
  if (recentFiles.value.length && !(await confirmDialog('确定要清空最近文件记录吗？'))) {
    return
  }
  try {
    const s = await backend.clearRecentFiles()
    recentFiles.value = Array.isArray(s?.recent_files) ? s.recent_files : []
    notifyOk('已清空最近记录')
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
      if (!(await confirmDiscard('打开'))) return
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
