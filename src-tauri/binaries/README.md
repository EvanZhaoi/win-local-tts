# ffmpeg 内置文件

此目录应包含 ffmpeg Windows x86_64 可执行文件。

## 下载方法

在有代理的电脑上下载：

```bash
# 方法 1: 从 GitHub releases 下载 (推荐)
curl -L -o ffmpeg.zip https://github.com/GyanD/codexffmpeg/releases/download/7.1/ffmpeg-7.1-full_build.zip
unzip ffmpeg.zip
# 找到 bin/ffmpeg.exe 复制到当前目录，重命名为 ffmpeg-x86_64-pc-windows-msvc.exe

# 方法 2: 从 BtbN FFmpeg Builds
curl -L -o ffmpeg.zip https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip
```

## 验证方法

下载后验证文件类型：
```bash
file ffmpeg-x86_64-pc-windows-msvc.exe
```

正确结果应该是：
```
ELF 64-bit LSB executable, x86-64
```

注意：如果显示 ASCII text 或 HTML，说明下载失败，不是真正的二进制文件。

## 重要说明

- 必须下载 Windows x86_64 版本 (不是 Linux 或 macOS)
- 文件名必须是 `ffmpeg-x86_64-pc-windows-msvc.exe`
- 文件必须可执行（不是 zip 压缩包）
- 打包时会通过 `externalBin` 和 `resources` 自动包含在安装包中