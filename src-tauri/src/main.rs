// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use tauri::{AppHandle, Manager, BaseDirectory};

/// 从 Tauri 资源目录或 Path 获取 ffmpeg 路径
fn get_ffmpeg_path(app: &AppHandle) -> Result<PathBuf, String> {
    // 优先使用 Tauri 内置资源 (bundle 目录下的 binaries)
    if let Ok(res_path) = app.path().resolve("binaries/ffmpeg-x86_64-pc-windows-msvc.exe", BaseDirectory::Resource) {
        if res_path.exists() {
            return Ok(res_path);
        }
    }

    // fallback: 从 PATH 查找
    let path_env = std::env::var("PATH").unwrap_or_default();
    for dir in path_env.split(';') {
        let ffmpeg = PathBuf::from(dir).join("ffmpeg.exe");
        if ffmpeg.exists() {
            return Ok(ffmpeg);
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
    let ps_result = Command::new("powershell.exe")
        .args([
            "-ExecutionPolicy", "Bypass",
            "-NoProfile",
            "-File",
            script_path.to_string_lossy().as_ref(),
            "-Text", &text,
            "-OutputPath", wav_path.to_string_lossy().as_ref(),
            "-Rate", &rate.to_string(),
            "-Volume", &volume.to_string(),
        ])
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
    let ffmpeg_result = Command::new(&ffmpeg_path)
        .args([
            "-y",
            "-i", wav_path.to_string_lossy().as_ref(),
            "-codec:a", "libmp3lame",
            "-b:a", "128k",
            mp3_path.to_string_lossy().as_ref(),
        ])
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![generate_speech, save_audio])
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