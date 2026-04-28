// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use tauri::{AppHandle, Manager};
use tauri::path::BaseDirectory;
use base64::{engine::general_purpose, Engine as _};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 从 Tauri 资源目录或 Path 获取 ffmpeg 路径
fn get_ffmpeg_path(app: &AppHandle) -> Result<PathBuf, String> {
    // 尝试多个可能的路径
    let candidates = [
        "binaries/ffmpeg-x86_64-pc-windows-msvc.exe",
        "binaries/ffmpeg.exe",
        "binaries/ffmpeg",
    ];

    for candidate in &candidates {
        if let Ok(res_path) = app.path().resolve(candidate, BaseDirectory::Resource) {
            if res_path.exists() {
                return Ok(res_path);
            }
        }
    }

    // fallback: 从 PATH 查找
    let path_env = std::env::var("PATH").unwrap_or_default();
    for dir in path_env.split(';') {
        let ffmpeg = PathBuf::from(dir).join("ffmpeg.exe");
        if ffmpeg.exists() {
            return Ok(ffmpeg);
        }
        // 也尝试不带 .exe 的版本
        let ffmpeg_no_ext = PathBuf::from(dir).join("ffmpeg");
        if ffmpeg_no_ext.exists() {
            return Ok(ffmpeg_no_ext);
        }
    }

    Err("未找到 ffmpeg.exe，请先下载并放入 src-tauri/binaries/ 目录\n下载地址: https://github.com/GyanD/codexffmpeg/releases/download/8.1/ffmpeg-8.1-full_build.zip\n解压后将 bin/ffmpeg.exe 重命名为 ffmpeg-x86_64-pc-windows-msvc.exe 放入 src-tauri/binaries/".to_string())
}

/// 生成唯一的临时文件名
fn generate_unique_filename(temp_dir: &PathBuf, prefix: &str, ext: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    temp_dir.join(format!("{}_{}.{}", prefix, timestamp, ext))
}

/// 写入 PowerShell 脚本文件
fn write_ps_script(script_path: &PathBuf) -> Result<(), String> {
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

#[tauri::command]
async fn generate_speech(
    app: AppHandle,
    text: String,
    rate: i32,
    volume: u32,
) -> Result<String, String> {
    // 参数验证
    if text.trim().is_empty() {
        return Err("文字不能为空".to_string());
    }

    if text.len() > 5000 {
        return Err("文字过长，最多 5000 字".to_string());
    }

    let rate = rate.max(-10).min(10);
    let volume = volume.max(0).min(100);

    // 创建临时目录
    let temp_dir = std::env::temp_dir().join("win_local_tts");
    fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;

    // 生成唯一的文件路径
    let wav_path = generate_unique_filename(&temp_dir, "tts", "wav");
    let mp3_path = generate_unique_filename(&temp_dir, "tts", "mp3");
    let script_path = generate_unique_filename(&temp_dir, "tts_script", "ps1");

    // 写入 PowerShell 脚本文件
    write_ps_script(&script_path)?;

    // 调用 PowerShell 执行脚本（参数传递 text，避免字符串拼接）
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
    #[cfg(target_os = "windows")]
    ps_command.creation_flags(CREATE_NO_WINDOW);

    let ps_result = ps_command
        .output()
        .map_err(|e| format!("执行 PowerShell 失败: {}", e))?;

    // 清理脚本文件
    let _ = fs::remove_file(&script_path);

    if !ps_result.status.success() {
        let stderr = String::from_utf8_lossy(&ps_result.stderr);
        return Err(format!("TTS 生成失败: {}", stderr));
    }

    if !wav_path.exists() {
        return Err("WAV 文件未生成，请检查 Windows TTS 是否可用".to_string());
    }

    // 调用 ffmpeg 转换
    let ffmpeg_path = get_ffmpeg_path(&app)?;
    let mut ffmpeg_command = Command::new(&ffmpeg_path);
    ffmpeg_command
        .args([
            "-y",
            "-i", wav_path.to_string_lossy().as_ref(),
            "-codec:a", "libmp3lame",
            "-b:a", "128k",
            mp3_path.to_string_lossy().as_ref(),
        ]);
    #[cfg(target_os = "windows")]
    ffmpeg_command.creation_flags(CREATE_NO_WINDOW);

    let ffmpeg_result = ffmpeg_command
        .output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    // 清理 WAV 文件
    let _ = fs::remove_file(&wav_path);

    if !ffmpeg_result.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_result.stderr);
        return Err(format!("MP3 转换失败: {}", stderr));
    }

    if !mp3_path.exists() {
        return Err("MP3 文件未生成".to_string());
    }

    Ok(mp3_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn save_audio(source: String, target: String) -> Result<(), String> {
    fs::copy(&source, &target).map_err(|e| format!("保存文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn read_audio_base64(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("读取音频失败: {}", e))?;
    let encoded = general_purpose::STANDARD.encode(bytes);
    Ok(encoded)
}

#[derive(serde::Serialize)]
struct SystemUser {
    username: String,
    computer: String,
}

#[tauri::command]
fn get_system_user() -> Result<SystemUser, String> {
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string());
    let computer = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());

    Ok(SystemUser {
        username,
        computer,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![generate_speech, save_audio, read_audio_base64, get_system_user])
        .setup(|app| {
            let temp_dir = std::env::temp_dir().join("win_local_tts");
            let _ = fs::create_dir_all(&temp_dir);
            println!("临时目录: {:?}", temp_dir);

            // 检查 ffmpeg 是否存在
            match get_ffmpeg_path(app.handle()) {
                Ok(path) => println!("找到 ffmpeg: {:?}", path),
                Err(e) => eprintln!("警告: {}", e),
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}

fn main() {
    run();
}