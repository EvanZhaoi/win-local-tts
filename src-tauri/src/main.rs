// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

fn get_ffmpeg_path() -> Result<PathBuf, String> {
    // 优先使用内置的 ffmpeg
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("无法获取 exe 所在目录")?;

    let bundled_ffmpeg = exe_dir.join("ffmpeg.exe");
    if bundled_ffmpeg.exists() {
        return Ok(bundled_ffmpeg);
    }

    // 开发环境：从 PATH 查找
    let path_result = std::env::var("PATH").unwrap_or_default();
    for dir in path_result.split(';') {
        let ffmpeg = PathBuf::from(dir).join("ffmpeg.exe");
        if ffmpeg.exists() {
            return Ok(ffmpeg);
        }
    }

    Err("未找到 ffmpeg，请确保已内置或添加到 PATH".to_string())
}

fn sanitize_param(param: &str) -> String {
    // 转义特殊字符以避免 PowerShell 注入
    param
        .replace("\\", "\\\\")
        .replace("\"", "`\"")
        .replace("$", "`$")
        .replace("'", "''")
}

#[tauri::command]
async fn generate_speech(
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
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let wav_path = temp_dir.join("temp_audio.wav");
    let mp3_path = temp_dir.join("output.mp3");

    // 清理旧文件
    let _ = fs::remove_file(&wav_path);
    let _ = fs::remove_file(&mp3_path);

    // PowerShell 脚本内容
    let safe_text = sanitize_param(&text);
    let safe_wav = sanitize_param(wav_path.to_string_lossy().as_ref());

    let ps_script = format!(
        r#"
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$synth.Rate = {}
$synth.Volume = {}
$synth.SetOutputToWaveFile("{}")
$synth.Speak("{}")
$synth.Dispose()
"#,
        rate, volume, safe_wav, safe_text
    );

    // 执行 PowerShell
    let ps_result = Command::new("powershell.exe")
        .args([
            "-ExecutionPolicy", "Bypass",
            "-NoProfile",
            "-Command", &ps_script
        ])
        .output()
        .map_err(|e| format!("执行 PowerShell 失败: {}", e))?;

    if !ps_result.status.success() {
        let stderr = String::from_utf8_lossy(&ps_result.stderr);
        return Err(format!("TTS 生成失败: {}", stderr));
    }

    if !wav_path.exists() {
        return Err("WAV 文件未生成".to_string());
    }

    // 调用 ffmpeg 转换
    let ffmpeg_path = get_ffmpeg_path()?;
    let ffmpeg_result = Command::new(&ffmpeg_path)
        .args([
            "-y",
            "-i", wav_path.to_string_lossy().as_ref(),
            "-codec:a", "libmp3lame",
            "-b:a", "128k",
            mp3_path.to_string_lossy().as_ref()
        ])
        .output()
        .map_err(|e| format!("ffmpeg 执行失败: {}", e))?;

    if !ffmpeg_result.status.success() {
        let stderr = String::from_utf8_lossy(&ffmpeg_result.stderr);
        return Err(format!("MP3 转换失败: {}", stderr));
    }

    if !mp3_path.exists() {
        return Err("MP3 文件未生成".to_string());
    }

    Ok(mp3_path.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![generate_speech])
        .setup(|app| {
            // 创建临时目录
            let temp_dir = std::env::temp_dir().join("win_local_tts");
            let _ = fs::create_dir_all(&temp_dir);
            println!("临时目录: {:?}", temp_dir);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}