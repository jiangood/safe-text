# SafeText · TXT 透明加密记事本

一个**绿色免安装**的小工具，用于加密单个 `.txt` 文件。加密后的文件**只有本软件能正常打开编辑**，用系统记事本、Notepad++、VS Code 等任何其他软件打开都是乱码。

> 技术栈：**Quasar (Vue 3) + Tauri 2**，加密在 Rust 侧
> 状态：方案锁定，开发实施中

---

## 一、核心目标

- 加密单个 `.txt` 文件，保留 `.txt` 后缀
- 只有本软件能正常打开 / 编辑
- 其他编辑器打开呈现乱码
- 密码错误拒绝打开并提示

**平台**：Windows / macOS / Linux 桌面 + Android。

---

## 二、功能需求

| 功能 | 说明 |
|---|---|
| 文件加密 | 选择 `.txt` → 输入密码 → 加密，保留 `.txt` 后缀 |
| 打开与编辑 | 选加密文件 → 输密码 → 解密显示明文；直接编辑 |
| 实时保存 | 保存即自动重新加密写回原文件，无需先解密再加密 |
| 新建 | 新建加密 txt，输入内容后设密码保存 |
| 另存为 | 加密写到新路径 |
| 修改密码 | 用新密码（新 salt + 密钥）重新加密写回 |
| 拖拽打开 | 文件拖入窗口 → 弹密码框（Android 用分享 Intent 替代） |
| 闲置自动锁定 | 默认 5 分钟，到时清空编辑区并丢弃密钥 |

---

## 三、安全要求

| 项目 | 要求 |
|---|---|
| 加密算法 | **AES-256-GCM**（公开标准，认证加密） |
| 密钥派生 | **Argon2id**（随机 salt） |
| 随机数 | 系统级安全随机数生成 salt / nonce |
| 完整性 | GCM 认证防篡改；文件头作 AAD |
| 文件格式 | 自定义头存 salt + nonce + 参数 + 版本号，其后密文 |
| 禁止 | 不自创加密算法，不硬编码密钥 |
| 运行方式 | 绿色免安装，桌面单 exe / Android APK |
| 密码 | 不存储、无后门；丢失即永久无法恢复 |

---

## 四、技术栈

| 层 | 选型 | 版本 |
|---|---|---|
| UI 框架 | Quasar（Vue 3 + Vite，`@quasar/app-vite`） | quasar 2.35 / app-vite 3.10 |
| 原生壳 | Tauri 2（Rust） | tauri 2.12 / tauri-build 2.7 |
| 文件对话框 | `tauri-plugin-dialog` | 2.8 |
| 文件读写 | `tauri-plugin-fs`（处理 Android `content://`） | 2.6 |
| 设置存储 | `tauri-plugin-store` | 2.5 |
| 对称加密 | `aes-gcm`（AES-256-GCM） | 0.11 |
| 密钥派生 | `argon2`（Argon2id） | 0.6 |
| 随机数 | `getrandom`（OS CSPRNG） | 0.4 |
| 内存擦除 | `zeroize` | 1.9 |
| 错误处理 | `anyhow` | 1.0 |

**关键决策**
- 加密放 **Rust**：可用 Argon2id（WebCrypto 无）、`zeroize` 可靠擦除、Android 复用同一份代码。
- 不保留"最近打开列表"（隐私），配置文件不保存任何文件路径。
- 中文字体由 WebView 自带渲染，无需额外处理。
- 设置存储：桌面便携优先（程序目录可写则写旁边，否则系统配置目录）。

---

## 五、文件格式 v1（48 字节固定头，大端）

```
偏移  长度  字段
0     4     Magic "STXT"
4     1     版本 = 0x01
5     1     KDF 标识 = 0x02 (Argon2id)
6     1     加密标识 = 0x01 (AES-256-GCM)
7     1     保留(0)
8     4     Argon2 m_cost (KiB, 默认 65536)
12    4     Argon2 t_cost (默认 3)
16    4     Argon2 p_cost (默认 1)
20    16    salt
36    12    nonce/IV
48    ..    密文 + 16 字节 GCM tag
```

- 前 48 字节作为 GCM 的 **AAD**，头部被篡改会解密失败
- 每次保存生成**新 nonce**；"修改密码"重新生成 salt + 密钥
- Magic 不符 → 提示"不是有效加密文件"；GCM 校验失败 → "密码错误或文件已损坏"

---

## 六、界面需求（极简）

- 主窗口一个多行文本框（Quasar `QInput type="textarea"` 占满布局）
- 工具栏 `QToolbar` + `QBtn`：新建 / 打开 / 保存 / 另存为 / 修改密码
- 密码框：`QDialog` + `QInput type="password"`（新建 / 改密二次确认）
- 标题 / 状态栏：文件名 + 已加密 + 未保存 `*`

---

## 七、跨平台处理

| 维度 | 桌面 | Android |
|---|---|---|
| 选文件 | 原生对话框（dialog 插件） | SAF（`text/plain`），返回 `content://` |
| 读 / 写 | fs 插件（路径） | fs 插件（content://） |
| 打开入口 | 拖拽文件 | 分享 Intent（`ACTION_VIEW` / `ACTION_SEND`） |
| 自动锁定 | 闲置计时 | 闲置计时 + `onPause/onStop` 清空明文 |
| 设置存储 | 便携优先，否则系统配置目录 | store 插件（应用私有目录） |

**文本处理**：按字节读入 → 校验头 → 解密 → UTF-8 解码；去除 BOM；**保留原文件 CRLF/LF 行尾**；非 UTF-8（如 GBK）给出明确提示，不静默损坏。

---

## 八、工程结构

```
safe-text/
  package.json               quasar.config.ts
  index.html
  src/
    App.vue
    pages/EditorPage.vue
    components/{Toolbar,PasswordDialog,StatusBar}.vue
    composables/useDocument.ts     # 会话状态：路径/密钥标记/脏标记/自动锁定
  src-tauri/
    Cargo.toml   tauri.conf.json   build.rs
    capabilities/default.json
    src/
      main.rs      # 桌面入口
      lib.rs       # Builder、注册插件与命令
      commands.rs  # 暴露给前端的命令
      session.rs   # 会话状态：路径 + Zeroizing 密钥 + 参数
      crypto.rs    # Argon2id + AES-256-GCM（含单元测试）
      format.rs    # 48 字节头编解码
      config.rs    # 设置（桌面便携优先）
  README.md
```

### 命令接口（Rust → 前端 `invoke`）

`new_document` / `open_document` / `save_document` / `save_document_as` / `change_password` / `lock` / `probe_file`

---

## 九、构建与验证

### 桌面

```bash
quasar dev                 # 浏览器 SPA，调 UI
quasar dev -m tauri        # Tauri 原生窗口
quasar build -m tauri      # 出桌面 exe
```

- Windows 需在 vcvars64 环境下构建（MSVC 14.51 + SDK 10.0.26100）。
- 可用 `+crt-static` 去掉 VC++ 运行库依赖。

### Android

```bash
tauri android init
tauri android build --apk
```

- 前置：Android SDK + NDK + JDK 21 + `cargo-ndk` + android rust targets。
- 无模拟器 / 真机时只能编译 APK，无法实测运行。

### 验证

1. `cargo test`：加解密往返、错误密码拒绝、篡改检测、Magic 校验、截断报错
2. 桌面实测：新建 → 保存 → 记事本打开乱码 → 本软件重开正常 → 改密码 → 重开正常
3. Android：编译产出 APK

---

## 十、已知风险

1. **Android 工具链**：需安装 SDK + NDK（数 GB）。
2. **Android 写回 `content://` 存在已知坑**（plugins-workspace #3356），可能需 `tauri-plugin-android-fs` 或补一小段 Kotlin/JNI。
3. **Argon2 内存参数**：低端机 64MiB 偏重，可给 Android 用较低参数（参数写在文件头，桌面仍可读）。

---

## 十一、安全提醒

- 密码丢失 = 文件**永久无法恢复**，软件不存密码、无后门
- 重要文件建议保留未加密备份
- 不要用来路不明的第三方"加密算法"
