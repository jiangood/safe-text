# SafeText · TXT 透明加密记事本

一个**绿色免安装**的小工具，用于加密单个 `.txt` 文件。加密后的文件**只有本软件能正常打开编辑**，用系统记事本、Notepad++、VS Code 等任何其他软件打开都是乱码。

> 技术栈：**Quasar (Vue 3 + Vite) + Tauri 2**，加密全部在 Rust 侧完成
> 状态：桌面端已实现并可构建（`cargo test` 通过，`quasar build` 通过）；Android 目标已保留代码路径

---

## 一、核心目标

- 加密单个 `.txt` 文件，保留 `.txt` 后缀
- 只有本软件能正常打开 / 编辑，其它编辑器打开呈现乱码
- 密码错误拒绝打开并提示
- 标准密码学，不自创算法、不硬编码密钥

**平台**：Windows / macOS / Linux 桌面 + Android（同一份 Rust 加密代码）。

---

## 二、功能

| 功能 | 说明 | 状态 |
|---|---|---|
| 新建 | 设密码 → 加密 → 另存为 `.txt` | ✅ |
| 打开 | 选加密文件 → 输密码 → 解密显示明文 | ✅ |
| 编辑 | 单多行文本框直接编辑 | ✅ |
| 保存 | 重新加密写回原文件（新 nonce，无需重输密码） | ✅ |
| 另存为 | 加密写到新路径 | ✅ |
| 修改密码 | 新 salt + 新密钥重新加密写回 | ✅ |
| 锁定 | 清空界面并丢弃密钥 | ✅ |
| 拖拽打开 | 文件拖入窗口 → 弹密码框 | ✅（桌面） |
| 闲置自动锁定 | 默认 5 分钟，到时清空并丢弃密钥 | ✅ |
| 键盘快捷键 | Ctrl+N / O / S / Shift+S / L | ✅ |

> 隐私：**不保留"最近打开列表"**，设置文件不含任何文件路径。

---

## 三、安全设计

| 项目 | 实现 |
|---|---|
| 加密算法 | **AES-256-GCM**（认证加密，密文尾部 16 字节 tag） |
| 密钥派生 | **Argon2id**，随机 16 字节 salt，默认 m=64 MiB / t=3 / p=1 |
| 随机数 | `getrandom`（操作系统 CSPRNG）生成 salt / nonce |
| 完整性 | 前 48 字节文件头作为 **GCM AAD**，篡改头部即解密失败 |
| 文件格式 | 固定 48 字节头（salt + nonce + 参数 + 版本），其后密文 |
| 内存安全 | 派生密钥用 `zeroize::Zeroizing` 包裹，锁定 / 退出即擦除 |
| 禁止项 | 不自创加密算法，不硬编码密钥，不保存密码 |
| 密码 | 不存储、无后门；丢失即**永久无法恢复** |

---

## 四、技术栈

| 层 | 选型 | 版本 |
|---|---|---|
| UI 框架 | Quasar（Vue 3 + Vite，`@quasar/app-vite`） | quasar 2.35 / app-vite 3.10 |
| 原生壳 | Tauri 2（Rust，`edition 2024`） | tauri 2.12 / tauri-build 2.7 |
| 文件对话框 | `tauri-plugin-dialog` | 2.8 |
| 文件读写 | `tauri-plugin-fs`（桌面 + Android `content://`） | 2.6 |
| 日志 | `tauri-plugin-log` | 2 |
| 对称加密 | `aes-gcm`（AES-256-GCM） | 0.11 |
| 密钥派生 | `argon2`（Argon2id） | 0.6 |
| 随机数 | `getrandom`（OS CSPRNG） | 0.4 |
| 内存擦除 | `zeroize` | 1 |
| 设置存储 | 自研便携优先 JSON（见 §八） | — |

**关键决策**
- 加密放 **Rust**：可用 Argon2id（WebCrypto 无）、`zeroize` 可靠擦除、桌面与 Android 复用同一份代码。
- **文件 I/O 放前端插件**：用 `dialog` + `fs` 插件读写，天然支持 Android `content://` URI；加密命令只收发字节，不碰文件系统（便于单测）。
- 拖拽打开由 Rust 命令 `read_file` 读取（拖入路径没有对话框授权 scope）。
- 不保留"最近打开列表"，配置不保存文件路径。

---

## 五、文件格式 v1（48 字节固定头，大端）

```
偏移  长度  字段
0     4     Magic "STXT"
4     1     版本 = 0x01
5     1     KDF 标识 = 0x01 (Argon2id)
6     1     加密标识 = 0x01 (AES-256-GCM)
7     1     保留(0)
8     4     Argon2 m_cost (KiB, 默认 65536)
12    4     Argon2 t_cost (默认 3)
16    4     Argon2 p_cost (默认 1)
20    16    salt
36    12    nonce / IV
48    ..    密文 + 16 字节 GCM tag
```

- 前 48 字节作为 GCM 的 **AAD**，头部被篡改会解密失败
- 每次保存生成**新 nonce**；"修改密码"重新生成 salt + 密钥
- Magic 不符 → `不是有效加密文件`；GCM 校验失败 → `密码错误或文件已损坏`
- 参数写在头里，将来调整默认强度仍可读旧文件

---

## 六、界面

- 顶部工具栏 `q-toolbar` + `q-btn`：新建 / 打开 / 保存 / 另存为 / 改密码 / 锁定
- 中间一个占满窗口的多行文本框（等宽字体）
- 底部状态栏：文件名、加密状态、保存状态、算法标识
- 密码框 `q-dialog`：新建 / 改密二次确认并校验长度；标题栏显示文件名 + `*`（未保存）

---

## 七、跨平台处理

| 维度 | 桌面 | Android |
|---|---|---|
| 选文件 | 原生对话框（dialog 插件） | SAF（`text/plain`），返回 `content://` |
| 读 / 写 | fs 插件（路径） | fs 插件（`content://`） |
| 打开入口 | 拖拽文件 | 分享 Intent（`ACTION_VIEW` / `ACTION_SEND`，待做） |
| 自动锁定 | 闲置计时 | 闲置计时 + `onPause/onStop`（待做） |
| 设置存储 | 便携优先，否则系统配置目录 | 应用配置目录 |

**文本处理**：按字节读入 → 校验头 → 解密 → UTF-8 解码。非 UTF-8（如 GBK）会明确提示"内容不是有效的文本"，不静默损坏。
**已知限制**：HTML `textarea` 会把 CRLF 规范化为 LF，因此编辑后保存可能改变行尾；BOM 会作为 `U+FEFF` 原样保留。

---

## 八、工程结构

```
safe-text/
  package.json          quasar.config.js      index.html
  src/
    App.vue
    pages/IndexPage.vue                 # 编辑器页面（工具栏 + 文本框 + 密码框 + 自动锁定 + 拖拽）
    services/backend.js                 # 调用 Rust 命令 & Tauri 插件（对话框 / 文件 / 拖拽）
    router/{index.js,routes.js}
    css/{app.scss,quasar.variables.scss}
  src-tauri/
    Cargo.toml  tauri.conf.json  build.rs
    capabilities/default.json
    src/
      main.rs      # 桌面入口
      lib.rs       # Builder：注册插件、命令、会话状态
      commands.rs  # 暴露给前端的命令
      session.rs   # 会话状态：Zeroizing 密钥 + salt + 参数
      crypto.rs    # Argon2id + AES-256-GCM（含单元测试）
      format.rs    # 48 字节头编解码
      settings.rs  # 设置（桌面便携优先）
      errors.rs    # 错误 → 中文提示
  README.md
```

### 命令接口（Rust → 前端 `invoke`）

| 命令 | 作用 |
|---|---|
| `new_document(password, text)` | 新加密文档，解锁会话，返回密文字节 |
| `open_document(password, data)` | 解密，解锁会话，返回明文 |
| `save_document(text)` | 用缓存密钥重新加密（新 nonce），返回密文字节 |
| `change_password(new_password, text)` | 新 salt+密钥重新加密，更新会话 |
| `lock()` | 清空并擦除会话密钥 |
| `probe_file(data)` | 结构探测，判断是否为 SafeText 文件 |
| `read_file(path)` | 按路径读文件（拖拽用） |
| `load_settings()` / `save_settings(settings)` | 读写设置 |

---

## 九、构建与验证

### 前置

- Node ≥ 22、Rust ≥ 1.90、Tauri 2 系统依赖
- Windows 需 MSVC 链接器（在 `vcvars64.bat` 环境下构建）

### 桌面

```bash
npm install
npm run tauri:dev      # 原生开发窗口（先跑 quasar dev 再起 Tauri）
npm run tauri:build    # 产出桌面安装包 / exe
npm run build          # 仅构建前端 SPA（输出 dist/spa）
```

`src-tauri/tauri.conf.json` 已配置：`beforeDevCommand = npm run dev`、`devUrl = http://localhost:9000`、`beforeBuildCommand = npm run build`、`frontendDist = ../dist/spa`。

### Android

```bash
npm run tauri -- android init
npm run tauri -- android build --apk
```

- 前置：Android SDK + NDK + JDK 21 + `cargo-ndk` + android rust targets。
- 无模拟器 / 真机时只能编译 APK，无法实测运行。

### 验证

1. `cargo test`（在 `src-tauri/`）：加解密往返、错误密码拒绝、篡改检测、头部篡改、Magic 校验、截断报错、空内容
2. 桌面实测：新建 → 保存 → 记事本打开乱码 → 本软件重开正常 → 改密码 → 重开正常
3. Android：编译产出 APK

---

## 十、已知风险 / 待办

1. **Android 工具链**未安装：需 SDK + NDK（数 GB）才能出 APK。
2. **Android 写回 `content://`** 存在插件已知坑（plugins-workspace #3356），如遇问题可换 `tauri-plugin-android-fs` 或补一小段 Kotlin/JNI。
3. **分享 Intent**（Android 从文件管理器直接打开）尚未实现。
4. **应用级 CSP** 目前关闭以保证 Tauri IPC 稳定；后续可在 `tauri.conf.json` 的 `app.security.csp` 收紧。
5. **CRLF 行尾**在编辑保存后可能被规范化为 LF（见 §七）。

---

## 十一、安全提醒

- 密码丢失 = 文件**永久无法恢复**，软件不存密码、无后门
- 重要文件建议保留未加密备份
- 不要用来路不明的第三方"加密算法"
