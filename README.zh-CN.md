# SafeText · TXT 透明加密记事本

[English](README.md) · **简体中文**

一个**绿色免安装**的小工具，用于加密单个 `.txt` 文件。加密后的文件**只有本软件能正常打开编辑**，用系统记事本、Notepad++、VS Code 等任何其他软件打开都是乱码。

> 技术栈：**Quasar (Vue 3 + Vite) + Tauri 2**，加密全部在 Rust 侧完成
> 状态：Windows / Android 已实现并接入 GitHub Actions 自动打包发布（`cargo test` 通过，桌面端本地构建通过）

![SafeText 主界面](docs/screenshot.png)

---

## 一、核心目标

- 加密单个 `.txt` 文件，保留 `.txt` 后缀
- 只有本软件能正常打开 / 编辑，其它编辑器打开呈现乱码
- 密码错误拒绝打开并提示
- 标准密码学，不自创算法、不硬编码密钥

**发布平台**：Windows（`.msi` / `-setup.exe`）+ Android（`.apk`），同一份 Rust 加密代码。

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
| 最近文件 | 默认关闭；启用后工具栏列出最近打开路径，可一键**清空** | ✅（桌面） |
| 设置 | 自动锁定分钟数 + "记住最近文件"开关 + 语言 | ✅ |
| 多语言 | 简体中文 / English，默认跟随系统，可在设置中手动切换 | ✅ |

> 隐私：**"最近文件"默认关闭、需手动开启**。开启后仅在本地设置文件保存文件路径；关闭开关或点击**清空**会立即抹除，因此默认情况下不保存任何路径。

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
- 拖拽 / 最近列表打开的路径没有对话框授权 scope，读写走 Rust 的 `read_file` / `write_file`；对话框打开的文件仍用 fs 插件。
- "最近文件"**默认关闭、需手动开启**；开启后仅把文件路径存进本地设置文件，关闭开关或**清空**即抹除。

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

主界面见文首截图。新建 / 改密时的密码框：

![新建文档的密码框](docs/screenshot-password.png)

- 顶部工具栏 `q-toolbar` + `q-btn`：新建 / 打开 / 最近 / 保存 / 另存为 / 改密码 / 锁定 / 设置
- "最近"下拉列出已记住的文件（默认关闭，在设置中开启），底部提供"清空最近记录"
- 设置对话框：自动锁定分钟数 + "记住最近打开的文件"开关
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
    i18n/{index.js,en-US.js,zh-CN.js}   # 零依赖 i18n（locale ref + t()）；词条都在这里
    boot/i18n.js                        # 挂载前应用已保存的语言
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
      errors.rs    # 错误 → 稳定错误码（JSON），由前端本地化
  README.md
  README.zh-CN.md
  docs/         # README 引用的界面截图
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
| `read_file(path)` | 按路径读文件（拖拽 / 最近文件用） |
| `write_file(path, data)` | 按路径写文件（拖拽 / 最近文件用） |
| `load_settings()` / `save_settings(settings)` | 读写设置（关闭"记住最近"时抹除路径） |
| `add_recent_file(path)` | 把路径记入（需手动开启的）最近列表，未开启时不生效 |
| `clear_recent_files()` | 清空全部最近路径 |

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

本地需要 Android SDK + NDK + JDK（Tauri 2 无需单独装 `cargo-ndk`）：

```bash
npm run tauri -- android init
npm run tauri -- android build --apk
```

- 前置：Android SDK + NDK + JDK 17+ + android rust targets（`aarch64-linux-android` 等，Tauri 会自动按需安装）。
- 无模拟器 / 真机时只能编译 APK，无法实测运行。
- 发布时由 GitHub Actions 自动完成 `android init` → `build --apk` → 用仓库 Secrets 中的签名密钥 `apksigner` 签名 → 上传到同一次 Release。

> ⚠️ Release 构建未经签名验证，装机会提示“未知来源”；Keep 好签名密钥（仓库 Secrets `ANDROID_KEYSTORE_*`），后续版本须用同一密钥才能覆盖安装。

### 验证

1. `cargo test`（在 `src-tauri/`）：加解密往返、错误密码拒绝、篡改检测、头部篡改、Magic 校验、截断报错、空内容
2. 桌面实测：新建 → 保存 → 记事本打开乱码 → 本软件重开正常 → 改密码 → 重开正常
3. Android：CI 编译并签名产出 APK（本机无 SDK/NDK 时无法本地实测，属已知限制）

---

## 十、已知风险 / 待办

1. **Android 真机行为未实测**：CI 能编译并签名产出 APK，但无模拟器/真机，运行期（尤其 `content://` 读写、自动锁定）需实机验证。
2. **Android 写回 `content://`** 存在插件已知坑（plugins-workspace #3356），如遇问题可换 `tauri-plugin-android-fs` 或补一小段 Kotlin/JNI。
3. **分享 Intent**（Android 从文件管理器直接打开）尚未实现。
4. **应用级 CSP** 目前关闭以保证 Tauri IPC 稳定；后续可在 `tauri.conf.json` 的 `app.security.csp` 收紧。
5. **CRLF 行尾**在编辑保存后可能被规范化为 LF（见 §七）。

---

## 十一、安全提醒

- 密码丢失 = 文件**永久无法恢复**，软件不存密码、无后门
- 重要文件建议保留未加密备份
- 不要用来路不明的第三方"加密算法"
