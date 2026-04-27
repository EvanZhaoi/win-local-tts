# WinLocalTTS - Windows 本地文字转语音工具

> 完全本地运行的 Windows 桌面应用，使用 Windows 系统自带语音生成 MP3，不调用任何云接口。

## 特性

- 🖥️ **Windows 桌面应用** - 可打包成 exe/msi 安装包
- 🔇 **完全离线** - 调用 Windows System.Speech，不依赖网络
- 🎵 **在线播放** - 生成后可直接播放
- 💾 **导出 MP3** - 支持保存为 MP3 文件
- ⚡ **内置 ffmpeg** - WAV 转 MP3 不需要用户安装 ffmpeg

## 技术栈

- **Tauri 2.0** - 桌面应用框架
- **React + TypeScript** - 前端界面
- **Rust** - 后端命令处理
- **Windows System.Speech** - 系统 TTS 引擎
- **ffmpeg** - 内置音频转换（已打包在应用中）

## 系统要求

- Windows 10/11
- 不需要安装 Python
- 不需要安装 ffmpeg（已内置）
- 不需要网络连接

## 重要：手动下载 ffmpeg

`src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe` 不会提交到 Git 仓库，需要手动下载。

### 下载步骤

1. 下载地址：https://github.com/GyanD/codexffmpeg/releases/download/7.1/ffmpeg-7.1-full_build.zip

2. 解压得到 `ffmpeg-*-full_build/bin/ffmpeg.exe`

3. 将 `ffmpeg.exe` 重命名为 `ffmpeg-x86_64-pc-windows-msvc.exe`

4. 放入 `src-tauri/binaries/` 目录

### 验证文件

```bash
ls -lh src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
```

正确大小约 **193MB**。

### 打包说明

- 打包时会通过 `externalBin` 和 `resources` 自动包含 ffmpeg
- 最终用户安装 exe/msi 后不需要单独下载 ffmpeg

## 开发运行

```bash
# 安装依赖
npm install

# 手动下载 ffmpeg 后启动开发服务器
npm run tauri dev
```

## 打包发布

```bash
# 1. 先下载 ffmpeg 放入 src-tauri/binaries/

# 2. 打包应用（生成 exe/msi）
npm run tauri build
```

打包完成后，安装包位于：
- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

## 使用说明

1. 打开应用
2. 输入要转换的文字（最多 5000 字）
3. 调整语速（-10 到 10）和音量（0 到 100）
4. 点击"生成语音"
5. 等待生成完成后，可在线播放或保存 MP3

## 常见问题

### Q: 提示 "未找到 ffmpeg"？
A: 请手动下载 ffmpeg 并放入 `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe`。

### Q: 生成的语音是英文的？
A: Windows 系统语音默认是英文，可以在 Windows 设置中将语音改为中文。

### Q: PowerShell 执行被拒绝？
A: 应用已使用 `-ExecutionPolicy Bypass` 参数，应该可以正常运行。如果仍有问题，请以管理员身份运行应用。

## 项目结构

```
win-local-tts/
├── src/                          # React 前端
│   ├── App.tsx                   # 主组件
│   ├── main.tsx                 # 入口
│   └── index.css                # 样式
├── src-tauri/                    # Tauri 后端
│   ├── src/
│   │   └── main.rs              # Rust 命令实现
│   ├── binaries/
│   │   └── ffmpeg-x86_64-pc-windows-msvc.exe  # ffmpeg (需手动下载)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── capabilities/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── .gitignore
└── README.md
```

## 验收标准

✅ 用户安装 exe 后可直接使用
✅ 不需要安装 ffmpeg（已内置）
✅ 不需要安装 Python
✅ 不需要联网
✅ 输入文字可生成语音
✅ 可播放
✅ 可保存 MP3