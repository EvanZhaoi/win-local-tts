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
- **ffmpeg** - 内置音频转换

## 系统要求

- Windows 10/11
- 不需要安装 Python
- 不需要安装 ffmpeg（已内置）
- 不需要网络连接

## 开发运行

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

## 打包发布

```bash
# 打包应用（生成 exe/msi）
npm run tauri build
```

打包完成后，安装包位于：
- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

## 内置 ffmpeg

ffmpeg 已放置在 `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe`，打包时会自动包含在安装包中。

用户不需要单独安装 ffmpeg。

## 使用说明

1. 打开应用
2. 输入要转换的文字（最多 5000 字）
3. 调整语速（-10 到 10）和音量（0 到 100）
4. 点击"生成语音"
5. 等待生成完成后，可在线播放或保存 MP3

## 常见问题

### Q: 生成的语音是英文的？
A: Windows 系统语音默认是英文，可以在 Windows 设置中将语音改为中文。

### Q: 提示 "未找到 ffmpeg"？
A: 请确保 `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe` 文件存在。

### Q: PowerShell 执行被拒绝？
A: 应用已使用 `-ExecutionPolicy Bypass` 参数，应该可以正常运行。如果仍有问题，请以管理员身份运行应用。

## 项目结构

```
win-local-tts/
├── src/                          # React 前端
│   ├── App.tsx                   # 主组件
│   ├── main.tsx                  # 入口
│   └── index.css                 # 样式
├── src-tauri/                    # Tauri 后端
│   ├── src/
│   │   └── main.rs               # Rust 命令实现
│   ├── binaries/
│   │   └── ffmpeg.exe            # 内置 ffmpeg
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── capabilities/
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

## API 命令

### generate_speech

生成语音并返回 MP3 文件路径。

**参数：**
- `text: String` - 要转换的文字
- `rate: i32` - 语速（-10 到 10）
- `volume: u32` - 音量（0 到 100）

**返回值：**
- 成功：MP3 文件路径
- 失败：错误信息

## License

MIT