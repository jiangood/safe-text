// Simplified Chinese messages. Keys are shared with en-US.js — keep the two
// files in sync when adding new strings.
export default {
  common: {
    ok: '确定',
    cancel: '取消',
    save: '保存',
    retry: '重试',
    gotIt: '知道了',
    confirmTitle: '确认',
    errorTitle: '出错了',
    processing: '处理中…',
    untitled: '未命名.txt'
  },

  toolbar: {
    new: '新建',
    open: '打开',
    recent: '最近',
    save: '保存',
    saveAs: '另存为',
    changePassword: '改密码',
    lock: '锁定',
    settings: '设置'
  },

  recent: {
    empty: '暂无最近文件',
    emptyHintEnabled: '打开过的文件会出现在这里',
    emptyHintDisabled: '在设置中启用“记住最近文件”后开始记录',
    clear: '清空最近记录',
    clearConfirm: '确定要清空最近文件记录吗？',
    cleared: '已清空最近记录'
  },

  status: {
    noDoc: '未打开文档',
    notOnDisk: '（尚未保存到磁盘）',
    unsaved: '未保存',
    saved: '已保存'
  },

  editor: {
    placeholderUnlocked: '在此输入内容，保存时将自动加密…',
    placeholderLocked: '点击“新建”创建加密文档，或“打开”已有加密 .txt 文件'
  },

  password: {
    label: '密码',
    newLabel: '新密码',
    confirmLabel: '确认密码',
    titleNew: '为新文档设置密码',
    titleOpen: '输入密码以打开',
    titleChange: '设置新密码',
    errEmpty: '请输入密码',
    errMismatch: '两次输入的密码不一致',
    errTooShort: '密码至少需要 6 个字符',
    warnNotSaved: '密码不会被保存，请务必牢记；遗失后无法恢复文件内容。'
  },

  dialog: {
    discard: '有未保存的修改，继续{action}将放弃这些修改。是否继续？',
    cannotOpen: '无法打开'
  },

  actions: {
    new: '新建',
    open: '打开',
    lock: '锁定'
  },

  settings: {
    title: '设置',
    autolock: '闲置自动锁定（分钟，0 表示关闭）',
    rememberRecent: '记住最近打开的文件',
    rememberRecentHint: '启用后仅在本地设置文件中保存文件路径，方便下次快速打开；关闭并保存将立即清除全部记录。',
    language: '语言',
    languageAuto: '自动（跟随系统）',
    languageZh: '简体中文',
    languageEn: 'English',
    saved: '设置已保存'
  },

  notify: {
    created: '已创建并保存',
    opened: '已打开',
    saved: '已保存',
    passwordChanged: '密码已修改',
    passwordChangedSaved: '密码已修改并保存',
    locked: '已锁定'
  },

  title: {
    encrypted: '已加密',
    unlocked: '未锁定'
  },

  dialogs: {
    encryptedText: '加密文本 (*.txt)',
    allFiles: '所有文件'
  },

  errors: {
    not_encrypted: '不是有效加密文件',
    bad_password: '密码错误或文件已损坏',
    no_session: '没有已解锁的文档',
    not_text: '解密成功，但内容不是有效的文本',
    unsupported_version: '不支持的文件版本：{detail}',
    unsupported_kdf: '不支持的口令派生算法：{detail}',
    unsupported_cipher: '不支持的加密算法：{detail}',
    kdf: '口令派生失败：{detail}',
    cipher: '加解密失败：{detail}',
    io: '文件操作失败：{detail}',
    unknown: '未知错误'
  }
}
