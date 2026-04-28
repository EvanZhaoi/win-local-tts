# WinLocalTTS - Windows 本地文字转语音工具

> 🎙️ 完全本地运行的 Windows 桌面应用，使用 Windows 系统自带语音生成 MP3，不调用任何云接口。

## 特性

- 🖥️ **Windows 桌面应用** - 可打包成 exe/msi 安装包
- 🔇 **完全离线** - 调用 Windows System.Speech，不依赖网络
- 🎵 **在线试听** - 生成后可直接在应用内试听
- 💾 **导出 MP3** - 支持保存为 MP3 文件
- ⚡ **内置 ffmpeg** - WAV 转 MP3 不需要用户安装 ffmpeg
- 📊 **使用统计** - 匿名统计工具使用情况（可选）

## 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 桌面框架 | Tauri 2.0 | Rust + WebView2 |
| 前端 | React + TypeScript | Vite 构建 |
| 后端 | Rust | Tauri 命令处理 |
| TTS 引擎 | Windows System.Speech | 系统自带 |
| 音频转换 | ffmpeg | 内置，无需安装 |

## 系统要求

- Windows 10/11（64位）
- 不需要安装 Python
- 不需要安装 ffmpeg（已内置）
- 不需要网络连接

## 快速开始

### 1. 下载 ffmpeg（首次开发时）

```bash
# 手动下载 ffmpeg 并放入指定目录
# 下载地址：https://github.com/GyanD/codexffmpeg/releases/download/8.1/ffmpeg-8.1-full_build.zip

# 解压后将 bin/ffmpeg.exe 重命名为 ffmpeg-x86_64-pc-windows-msvc.exe
# 放入 src-tauri/binaries/ 目录
```

### 2. 开发运行

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 3. 打包发布

```bash
# 打包应用（生成 exe/msi）
npm run tauri build
```

打包完成后，安装包位于：
- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

## 使用说明

1. 打开应用
2. 输入要转换的文字（最多 5000 字）
3. 调整语速（-10 到 10）和音量（0 到 100）
4. 点击"生成 MP3"
5. 等待生成完成后，可试听或保存 MP3

## 项目结构

```
win-local-tts/
├── src/                          # React 前端源码
│   ├── App.tsx                   # 主组件（包含所有业务逻辑）
│   ├── main.tsx                 # React 入口
│   └── index.css                # 全局样式
├── src-tauri/                    # Tauri/Rust 后端
│   ├── src/
│   │   └── main.rs              # Rust 命令实现
│   ├── binaries/
│   │   └── ffmpeg-x86_64-pc-windows-msvc.exe  # ffmpeg（需手动下载）
│   ├── Cargo.toml               # Rust 依赖配置
│   ├── tauri.conf.json         # Tauri 配置
│   └── capabilities/           # Tauri 权限配置
├── package.json                 # Node.js 依赖
├── tsconfig.json                # TypeScript 配置
├── vite.config.ts              # Vite 配置
├── index.html                  # HTML 入口
├── LICENSE                     # MIT 许可证
└── README.md                   # 本文件
```

## 配置说明

### 使用记录上报（可选）

应用默认关闭使用记录上报功能。如需启用，修改 `src/App.tsx` 中的配置：

```typescript
// 接口地址（替换为真实地址）
const REPORT_API_URL = "https://your-server.example.com/api/tts/usage-report";
```

上报内容（不包含用户输入文字或音频内容）：
```json
{
  "user": {
    "id": "windows-username",
    "name": "windows-username",
    "computer": "COMPUTER-NAME"
  },
  "event": "GENERATE_MP3",
  "usedAt": "2026-01-01T10:00:00.000Z",
  "client": {
    "app": "win-local-tts",
    "version": "1.0.0",
    "platform": "windows"
  }
}
```

### 图标自定义

应用图标位于 `src-tauri/icons/` 目录，包含以下文件：
- `icon.ico` - Windows 图标
- `icon.icns` - macOS 图标
- `app-icon.png` - 源图标（用于生成其他尺寸）
- `32x32.png`, `128x128.png`, `128x128@2x.png` - 各尺寸 PNG

## 常见问题

### Q: 提示 "未找到 ffmpeg"？
A: 请手动下载 ffmpeg 并放入 `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe`。

### Q: 生成的语音是英文的？
A: Windows 系统语音默认是英文，可在 Windows 设置中将默认语音改为中文。

### Q: PowerShell 执行被阻止？
A: 应用已使用 `-ExecutionPolicy Bypass` 参数，应该可以正常运行。如果仍有问题，请以管理员身份运行应用。

### Q: 生成的语音没有声音？
A: 请检查 Windows 系统音量设置，以及应用的音量参数。

## 许可证

本项目采用 MIT 许可证，详见 [LICENSE](LICENSE) 文件。

## 第三方依赖

| 依赖 | 版本 | 许可证 | 用途 |
|------|------|--------|------|
| Tauri | 2.0 | MIT | 桌面应用框架 |
| React | 18.x | MIT | UI 框架 |
| ffmpeg | 8.1 | LGPL/GPL | 音频转换 |
| Windows System.Speech | 内置 | - | 系统 TTS |

## 图标说明

本项目应用图标为自主设计的简化麦克风图标：
- 背景色: #00d9ff（青色）
- 前景色: #FFFFFF（白色）
- 设计理念参考 Tabler Icons（MIT 许可证）

## 贡献

欢迎提交 Issue 和 Pull Request！
