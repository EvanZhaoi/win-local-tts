// Prevents additional console window on Windows in release
// This attribute tells Windows to hide the console window when running in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// ============================================================================
// 导入 Rust 标准库和其他依赖
// ============================================================================

use std::fs;              // 文件系统操作：读写文件、创建目录等
use std::path::PathBuf;   // 路径类型，用于处理文件路径
use std::process::Command; // 进程管理，用于执行外部命令（PowerShell、ffmpeg）
use std::time::SystemTime; // 系统时间，用于生成唯一文件名
use tauri::{AppHandle, Manager}; // Tauri 框架核心：应用句柄和管理器
use tauri::path::BaseDirectory;   // Tauri 路径 API 的基础目录枚举
use base64::{engine::general_purpose, Engine as _}; // Base64 编码，用于音频文件传输

// ============================================================================
// Windows 平台专用代码
// ============================================================================

// 仅在 Windows 平台编译：导入 Windows 进程创建标志
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Windows 平台专用：CREATE_NO_WINDOW 标志
/// 设为 0x08000000 使得创建的子进程不显示控制台窗口
/// 这是解决 Windows 上弹出终端窗口问题的关键
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ============================================================================
// 常量定义
// ============================================================================

/// 文字转语音的最大字符数限制
/// 避免生成过长的音频文件
const MAX_TEXT_LENGTH: usize = 5000;

/// TTS 语速范围：-10（最慢）到 10（最快）
const RATE_MIN: i32 = -10;
const RATE_MAX: i32 = 10;

/// TTS 音量范围：0（静音）到 100（最大音量）
const VOLUME_MIN: u32 = 0;
const VOLUME_MAX: u32 = 100;

// ============================================================================
// 函数：获取 ffmpeg 可执行文件路径
// ============================================================================

/// 从 Tauri 资源目录或系统 PATH 中查找 ffmpeg 可执行文件
///
/// 查找优先级：
/// 1. Tauri 资源目录中的 ffmpeg-x86_64-pc-windows-msvc.exe
/// 2. Tauri 资源目录中的 ffmpeg.exe
/// 3. PATH 环境变量中的 ffmpeg.exe
///
/// # Arguments
/// * `app` - Tauri 应用句柄，用于访问应用资源
///
/// # Returns
/// * `Ok(PathBuf)` - ffmpeg 可执行文件的完整路径
/// * `Err(String)` - 未找到 ffmpeg 时的错误信息
fn get_ffmpeg_path(app: &AppHandle) -> Result<PathBuf, String> {
    // 优先在 Tauri 资源目录中查找
    let candidates = [
        "binaries/ffmpeg-x86_64-pc-windows-msvc.exe", // 推荐的带完整构建的版本
        "binaries/ffmpeg.exe",                         // 通用名称
        "binaries/ffmpeg",                             // 无扩展名版本
    ];

    for candidate in &candidates {
        // 尝试解析资源路径
        if let Ok(res_path) = app.path().resolve(candidate, BaseDirectory::Resource) {
            if res_path.exists() {
                return Ok(res_path);
            }
        }
    }

    // Fallback: 从 PATH 环境变量中查找
    let path_env = std::env::var("PATH").unwrap_or_default();
    for dir in path_env.split(';') {
        // 尝试带 .exe 扩展名的版本
        let ffmpeg = PathBuf::from(dir).join("ffmpeg.exe");
        if ffmpeg.exists() {
            return Ok(ffmpeg);
        }
        // 尝试不带扩展名的版本
        let ffmpeg_no_ext = PathBuf::from(dir).join("ffmpeg");
        if ffmpeg_no_ext.exists() {
            return Ok(ffmpeg_no_ext);
        }
    }

    // 未找到 ffmpeg，返回详细错误信息
    Err("未找到 ffmpeg.exe，请先下载并放入 src-tauri/binaries/ 目录\n下载地址: https://github.com/GyanD/codexffmpeg/releases/download/8.1/ffmpeg-8.1-full_build.zip\n解压后将 bin/ffmpeg.exe 重命名为 ffmpeg-x86_64-pc-windows-msvc.exe 放入 src-tauri/binaries/".to_string())
}

// ============================================================================
// 函数：生成唯一的临时文件名
// ============================================================================

/// 生成带时间戳的唯一文件名，避免文件冲突
///
/// # Arguments
/// * `temp_dir` - 临时文件目录
/// * `prefix`   - 文件名前缀（如 "tts"）
/// * `ext`      - 文件扩展名（如 "wav", "mp3"）
///
/// # Returns
/// 格式: {prefix}_{timestamp}.{ext}
/// 例如: tts_1714292400000.wav
fn generate_unique_filename(temp_dir: &PathBuf, prefix: &str, ext: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    temp_dir.join(format!("{}_{}.{}", prefix, timestamp, ext))
}

// ============================================================================
// 函数：写入 PowerShell 脚本文件
// ============================================================================

/// 将 PowerShell TTS 脚本写入临时文件
///
/// 脚本使用 Windows System.Speech 命名空间进行文字转语音
///
/// # Arguments
/// * `script_path` - 脚本文件路径
///
/// # Returns
/// * `Ok(())` - 写入成功
/// * `Err(String)` - 写入失败时的错误信息
fn write_ps_script(script_path: &PathBuf) -> Result<(), String> {
    // PowerShell 脚本内容：
    // - 使用 System.Speech.Synthesis.SpeechSynthesizer
    // - 支持语速(Rate)和音量(Volume)参数
    // - 输出为 WAV 格式
    let ps_content = r#"param(
    [string]$Text,
    [string]$OutputPath,
    [int]$Rate,
    [int]$Volume
)

Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$synth.Rate = $Rate
$synth.Volume = $Volume
$synth.SetOutputToWaveFile($OutputPath)
$synth.Speak($Text)
$synth.Dispose()
"#;

    fs::write(script_path, ps_content).map_err(|e| format!("写入脚本失败: {}", e))
}

// ============================================================================
// Tauri 命令：generate_speech - 生成语音
// ============================================================================

/// 将文字转换为语音 MP3 文件
///
/// 完整流程：
/// 1. 参数验证（文字内容、长度、语速、音量范围）
/// 2. 创建临时目录和文件路径
/// 3. 写入 PowerShell 脚本
/// 4. 调用 PowerShell 执行 TTS，生成 WAV 文件
/// 5. 调用 ffmpeg 将 WAV 转换为 MP3
/// 6. 清理临时文件，返回 MP3 路径
///
/// # Arguments
/// * `app`   - Tauri 应用句柄
/// * `text`  - 要转换的文字内容
/// * `rate`  - 语速（-10 到 10）
/// * `volume` - 音量（0 到 100）
///
/// # Returns
/// * `Ok(String)` - 生成的 MP3 文件完整路径
/// * `Err(String)` - 错误信息
#[tauri::command]
async fn generate_speech(
    app: AppHandle,
    text: String,
    rate: i32,
    volume: u32,
) -> Result<String, String> {
    // -------------------- 参数验证 --------------------
    
    // 检查文字是否为空
    if text.trim().is_empty() {
        return Err("文字不能为空".to_string());
    }

    // 检查文字长度
    if text.len() > MAX_TEXT_LENGTH {
        return Err(format!("文字过长，最多 {} 字", MAX_TEXT_LENGTH));
    }

    // 限制语速和音量范围
    let rate = rate.max(RATE_MIN).min(RATE_MAX);
    let volume = volume.max(VOLUME_MIN).min(VOLUME_MAX);

    // -------------------- 创建临时文件 --------------------
    
    // 在系统临时目录中创建应用专用文件夹
    let temp_dir = std::env::temp_dir().join("win_local_tts");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;

    // 生成唯一文件路径
    // WAV 文件：临时文件，转换后删除
    // MP3 文件：最终产物，返回给前端
    // PS1 文件：PowerShell 脚本，执行后删除
    let wav_path = generate_unique_filename(&temp_dir, "tts", "wav");
    let mp3_path = generate_unique_filename(&temp_dir, "tts", "mp3");
    let script_path = generate_unique_filename(&temp_dir, "tts_script", "ps1");

    // -------------------- 执行 TTS --------------------
    
    // 写入 PowerShell 脚本到临时文件
    write_ps_script(&script_path)?;

    // 构建 PowerShell 命令
    // -ExecutionPolicy Bypass: 绕过执行策略限制
    // -NoProfile: 不加载 PowerShell 配置文件，加快启动
    // -File: 指定脚本文件路径，参数通过 -Xxx 传递
    let mut ps_command = Command::new("powershell.exe");
    ps_command
        .args([
            "-ExecutionPolicy", "Bypass",
            "-NoProfile",
            "-File",
            script_path.to_string_lossy().as_ref(),
            "-Text", &text,
            "-OutputPath", wav_path.to_string_lossy().as_ref(),
            "-Rate", &rate.to_string(),
            "-Volume", &volume.to_string(),
        ]);
    
    // Windows 平台：隐藏控制台窗口，不弹出终端
    #[cfg(target_os = "windows")]
    ps_command.creation_flags(CREATE_NO_WINDOW);

    // 执行 PowerShell 脚本
    let ps_result = ps_command
        .output()
        .map_err(|e| format!("执行 PowerShell 失败: {}", e))?;

    // 清理脚本文件
    let _ = fs::remove_file(&script_path);

    // -------------------- 检查 TTS 结果 --------------------
    
    // 检查 PowerShell 是否成功执行
    if !ps_result.status.success() {
        let stderr = String::from_utf8_lossy(&ps_result.stderr);
        return Err(format!("TTS 生成失败: {}", stderr));
    }

    // 检查 WAV 文件是否生成
    if !wav_path.exists() {
        return Err("WAV 文件未生成，请检查 Windows TTS 是否可用".to_string());
    }

    // -------------------- 转换为 MP3 --------------------
    
    // 获取 ffmpeg 路径
    let ffmpeg_path = get_ffmpeg_path(&app)?;

    // 构建 ffmpeg 命令
    // -y: 自动覆盖输出文件
    // -i: 输入文件（WAV）
    // -codec:a libmp3lame: 使用 LAME 编码器转换为 MP3
    // -b:a 128k: 音频比特率 128kbps
    let mut ffmpeg_command = Command::new(&ffmpeg_path);
    ffmpeg_command
        .args([
            "-y",                                      // 自动覆盖
            "-i", wav_path.to_string_lossy().as_ref(), // 输入文件
            "-codec:a", "libmp3lame",                  // MP3 编码器
            "-b:a", "128k",                            // 比特率
            mp3_path.to_string_lossy().as_ref(),       // 输出文件
        ]);
    
    // Windows 平台：隐藏控制台窗口
    #[cfg(target_os = "windows")]
    ffmpeg_command.creation_flags(CREATE_NO_WINDOW);

    // 执行 ffmpeg 转换
    let ffmpeg_result = ffmpeg_command
        .output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    // 清理 WAV 文件
    let _ = fs::remove_file(&wav_path);

    // -------------------- 检查转换结果 --------------------
    
    if !ffmpeg_result.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_result.stderr);
        return Err(format!("MP3 转换失败: {}", stderr));
    }

    if !mp3_path.exists() {
        return Err("MP3 文件未生成".to_string());
    }

    // 返回 MP3 文件路径
    Ok(mp3_path.to_string_lossy().to_string())
}

// ============================================================================
// Tauri 命令：save_audio - 保存音频文件
// ============================================================================

/// 将生成的音频文件复制到用户指定的位置
///
/// # Arguments
/// * `source` - 源文件路径（临时目录中的 MP3）
/// * `target` - 目标文件路径（用户选择的位置）
///
/// # Returns
/// * `Ok(())` - 保存成功
/// * `Err(String)` - 保存失败时的错误信息
#[tauri::command]
async fn save_audio(source: String, target: String) -> Result<(), String> {
    fs::copy(&source, &target).map_err(|e| format!("保存文件失败: {}", e))?;
    Ok(())
}

// ============================================================================
// Tauri 命令：read_audio_base64 - 读取音频文件为 Base64
// ============================================================================

/// 读取音频文件并转换为 Base64 编码
///
/// 用于前端通过 data URL 方式播放音频，避免文件路径权限问题
///
/// # Arguments
/// * `path` - 音频文件路径
///
/// # Returns
/// * `Ok(String)` - Base64 编码的音频数据
/// * `Err(String)` - 读取失败时的错误信息
#[tauri::command]
async fn read_audio_base64(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("读取音频失败: {}", e))?;
    let encoded = general_purpose::STANDARD.encode(bytes);
    Ok(encoded)
}

// ============================================================================
// 结构体和命令：get_system_user - 获取系统用户信息
// ============================================================================

/// 系统用户信息结构体
/// 用于使用记录上报时标识用户身份
#[derive(serde::Serialize)]
struct SystemUser {
    /// Windows 登录用户名
    username: String,
    /// 计算机名称（机器名）
    computer: String,
}

/// 获取当前 Windows 系统用户信息
///
/// 从环境变量读取：
/// - USERNAME: Windows 登录账号名
/// - COMPUTERNAME: 计算机名称
///
/// # Returns
/// * `Ok(SystemUser)` - 包含用户名和计算机名的结构体
/// * `Err(String)` - 获取失败时的错误信息（通常不会发生）
#[tauri::command]
fn get_system_user() -> Result<SystemUser, String> {
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string());
    let computer = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());

    Ok(SystemUser {
        username,
        computer,
    })
}

// ============================================================================
// Tauri 应用入口
// ============================================================================

/// Tauri 应用运行时配置
/// 定义插件、命令处理器和应用初始化逻辑
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 初始化文件对话框插件（用于"保存 MP3"功能）
        .plugin(tauri_plugin_dialog::init())
        // 注册所有 Tauri 命令
        .invoke_handler(tauri::generate_handler![
            generate_speech,      // 文字转语音
            save_audio,           // 保存音频文件
            read_audio_base64,    // 读取音频为 Base64
            get_system_user       // 获取系统用户信息
        ])
        // 应用初始化回调
        .setup(|app| {
            // 创建临时目录
            let temp_dir = std::env::temp_dir().join("win_local_tts");
            let _ = fs::create_dir_all(&temp_dir);
            println!("临时目录: {:?}", temp_dir);

            // 检查 ffmpeg 是否可用
            match get_ffmpeg_path(app.handle()) {
                Ok(path) => println!("找到 ffmpeg: {:?}", path),
                Err(e) => eprintln!("警告: {}", e),
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}

/// 应用入口点
/// 调用 run() 函数启动应用
fn main() {
    run();
}
