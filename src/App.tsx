/**
 * ============================================================================
 * WinLocalTTS - Windows 本地文字转语音工具
 * 前端 React 组件
 * ============================================================================
 * 
 * 功能：
 * - 文字输入和参数调节（语速、音量、音色）
 * - 调用 Rust 后端生成 MP3 音频
 * - 音频预览（base64 data URL）
 * - 保存 MP3 到用户指定位置
 * - 使用记录上报（匿名统计）
 * 
 * 作者：Evan Zhao
 * 许可证：MIT
 */

// ============================================================================
// React 和 Tauri 依赖
// ============================================================================

import { useState, useEffect } from "react";  // React 状态管理和副作用
import { invoke } from "@tauri-apps/api/core";  // Tauri 命令调用
import { save } from "@tauri-apps/plugin-dialog";  // Tauri 文件保存对话框

// ============================================================================
// 常量定义
// ============================================================================

/** 文字输入的最大字符数 */
const MAX_TEXT_LENGTH = 5000;

/** 应用版本号（用于使用记录上报） */
const APP_VERSION = "1.0.0";

// ============================================================================
// 使用记录上报配置
// ============================================================================

/**
 * 使用记录上报接口地址
 * 
 * 替换为真实地址后，上报会发送到业务服务器
 */
const REPORT_API_URL = "https://your-server.example.com/api/tts/usage-report";

// ============================================================================
// 工具函数
// ============================================================================

/**
 * 获取当前 Windows 登录用户和计算机信息
 * 
 * 调用 Rust 后端的 get_system_user 命令获取：
 * - username: Windows 登录账号名
 * - computer: 计算机名称（机器名）
 * 
 * @returns 包含用户信息的对象
 */
async function getCurrentUser(): Promise<{
  id: string;
  name: string;
  computer: string;
}> {
  try {
    // 调用 Rust 后端命令
    const sys = await invoke<{
      username: string;
      computer: string;
    }>("get_system_user");
    
    // 返回用户信息对象
    return {
      id: sys.username,
      name: sys.username,
      computer: sys.computer,
    };
  } catch (e) {
    // 获取失败时返回默认值，不影响主流程
    console.warn("获取系统用户失败，使用默认值", e);
    return {
      id: "unknown",
      name: "unknown",
      computer: "unknown",
    };
  }
}

/**
 * 上报使用记录到业务服务器
 * 
 * 重要：此函数完全异步，不会阻塞本地功能
 * 上报失败只打印警告，不影响用户体验
 * 
 * 上报内容：
 * - user: 用户信息（用户名、计算机名）
 * - event: 事件类型（GENERATE_MP3）
 * - usedAt: 时间戳（ISO 8601 格式）
 * - client: 客户端信息（应用名、版本、平台）
 * 
 * 不上传的内容：用户输入文字、音频内容、文件路径
 */
async function reportUsage(): Promise<void> {
  try {
    // 获取当前用户信息
    const user = await getCurrentUser();
    
    // 发送 POST 请求到业务服务器
    await fetch(REPORT_API_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        user,
        event: "GENERATE_MP3",
        usedAt: new Date().toISOString(),
        client: {
          app: "win-local-tts",
          version: APP_VERSION,
          platform: "windows",
        },
      }),
    });
  } catch (error) {
    // 上报失败不影响本地功能，只打印警告
    console.warn("使用记录上报失败，但不影响本地功能", error);
  }
}

// ============================================================================
// React 组件
// ============================================================================

function App() {
  // -------------------- 状态定义 --------------------
  
  /** 用户输入的文字 */
  const [text, setText] = useState("");
  
  /** 语速：-10（最慢）到 10（最快），默认 0 */
  const [rate, setRate] = useState(0);
  
  /** 音量：0（静音）到 100（最大），默认 100 */
  const [volume, setVolume] = useState(100);
  
  /** 已安装的音色列表 */
  const [voices, setVoices] = useState<string[]>([]);
  
  /** 当前选中的音色 */
  const [voice, setVoice] = useState("");
  
  /** 状态提示文字 */
  const [status, setStatus] = useState("");
  
  /** 是否正在生成（防止重复点击） */
  const [isGenerating, setIsGenerating] = useState(false);
  
  /** 生成的 MP3 文件路径 */
  const [mp3Path, setMp3Path] = useState("");
  
  /** 音频预览的 base64 data URL */
  const [audioUrl, setAudioUrl] = useState("");

  // -------------------- 副作用：加载音色列表 --------------------

  /**
   * 应用启动时获取系统已安装的音色列表
   */
  useEffect(() => {
    invoke<string[]>("get_installed_voices")
      .then((list) => {
        setVoices(list);
        // 如果有音色列表，默认选择第一个
        if (list.length > 0) {
          setVoice(list[0]);
        }
      })
      .catch((err) => {
        console.warn("获取系统音色失败", err);
      });
  }, []);

  // -------------------- 事件处理函数 --------------------

  /**
   * 清空按钮点击处理
   * 重置所有输入状态
   */
  const handleClear = () => {
    setText("");
    setMp3Path("");
    setAudioUrl("");
    setStatus("");
  };

  /**
   * 生成 MP3 按钮点击处理
   * 
   * 完整流程：
   * 1. 参数验证（文字不能为空、长度限制）
   * 2. 调用 Rust 后端 generate_speech 生成 MP3（传入音色参数）
   * 3. 调用 Rust 后端 read_audio_base64 读取音频为 base64
   * 4. 更新状态，显示预览和保存按钮
   * 5. 异步上报使用记录
   */
  const handleGenerate = async () => {
    // 参数验证
    if (!text.trim()) {
      setStatus("请输入要转换的文字");
      return;
    }

    if (text.length > MAX_TEXT_LENGTH) {
      setStatus(`文字过长，最多 ${MAX_TEXT_LENGTH} 字`);
      return;
    }

    // 开始生成
    setIsGenerating(true);
    setStatus("");
    setMp3Path("");
    setAudioUrl("");

    try {
      // -------------------- 调用后端生成语音 --------------------
      
      const result = await invoke<string>("generate_speech", {
        text: text,
        rate: rate,
        volume: volume,
        voice: voice || null,  // 如果没有选择音色，传 null 使用系统默认
      });

      setMp3Path(result);

      // -------------------- 读取音频用于预览 --------------------
      
      const base64 = await invoke<string>("read_audio_base64", {
        path: result,
      });
      
      // 设置 audio 标签的 src 为 base64 data URL
      setAudioUrl(`data:audio/mpeg;base64,${base64}`);

      // 更新状态提示
      setStatus("语音生成成功，可以试听或保存 MP3");

      // -------------------- 异步上报使用记录 --------------------
      // 不 await，不阻塞后续操作
      reportUsage();
      
    } catch (err) {
      // 生成失败
      setStatus(`生成失败: ${err}`);
    } finally {
      setIsGenerating(false);
    }
  };

  /**
   * 保存 MP3 按钮点击处理
   * 
   * 流程：
   * 1. 打开系统文件保存对话框
   * 2. 用户选择保存位置后，调用 Rust 后端复制文件
   */
  const handleSave = async () => {
    // 没有生成的 MP3 时不执行
    if (!mp3Path) return;

    try {
      // 打开文件保存对话框
      const targetPath = await save({
        defaultPath: "output.mp3",  // 默认文件名
        filters: [
          { name: "MP3", extensions: ["mp3"] }
        ],
      });

      // 用户取消时 targetPath 为 null
      if (targetPath) {
        // 调用 Rust 后端复制文件
        await invoke("save_audio", {
          source: mp3Path,
          target: targetPath,
        });
        setStatus(`已保存到: ${targetPath}`);
      }
    } catch (err) {
      setStatus(`保存失败: ${err}`);
    }
  };

  /**
   * 根据状态文字返回对应的 CSS 类名
   * 
   * @returns CSS 类名：status-success（成功）、status-error（失败）、status-info（信息）
   */
  const getStatusClass = () => {
    if (status.includes("成功")) return "status-success";
    if (status.includes("失败")) return "status-error";
    if (isGenerating || status.includes("生成中")) return "status-info";
    return "status-info";
  };

  // -------------------- 渲染 --------------------
  
  return (
    <div className="container">
      <div className="card">
        {/* 标题区域 */}
        <div className="header">
          <h1>Windows 本地文字转语音</h1>
          <p>完全离线 · 使用系统语音 · 本地生成 MP3</p>
        </div>

        {/* 文字输入区域 */}
        <div className="form-group">
          <div className="form-label">
            <span className="form-label-text">输入文字</span>
          </div>
          
          {/* 清空按钮 */}
          <div className="clear-row">
            <button className="btn-clear" onClick={handleClear}>
              清空
            </button>
          </div>
          
          {/* 文字输入框 */}
          <textarea
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="请输入要转换为语音的文字..."
            maxLength={MAX_TEXT_LENGTH}
          />
          
          {/* 字符计数 */}
          <div className="chars-count">
            {text.length} / {MAX_TEXT_LENGTH}
          </div>
        </div>

        {/* 音色选择区域 */}
        <div className="form-group">
          <div className="form-label">
            <span className="form-label-text">音色</span>
          </div>
          <select
            className="voice-select"
            value={voice}
            onChange={(e) => setVoice(e.target.value)}
            disabled={voices.length === 0}
          >
            {voices.length === 0 ? (
              <option value="">使用系统默认音色</option>
            ) : (
              voices.map((v) => (
                <option key={v} value={v}>
                  {v}
                </option>
              ))
            )}
          </select>
        </div>

        {/* 语速控制区域 */}
        <div className="form-group">
          <div className="form-label">
            <span className="form-label-text">语速</span>
            <span className="form-label-value">{rate}</span>
          </div>
          <div className="range-control">
            <span className="range-label">慢</span>
            <input
              type="range"
              min="-10"
              max="10"
              step="1"
              value={rate}
              onChange={(e) => setRate(Number(e.target.value))}
            />
            <span className="range-label">快</span>
          </div>
        </div>

        {/* 音量控制区域 */}
        <div className="form-group">
          <div className="form-label">
            <span className="form-label-text">音量</span>
            <span className="form-label-value">{volume}%</span>
          </div>
          <div className="range-control">
            <span className="range-label">低</span>
            <input
              type="range"
              min="0"
              max="100"
              step="1"
              value={volume}
              onChange={(e) => setVolume(Number(e.target.value))}
            />
            <span className="range-label">高</span>
          </div>
        </div>

        {/* 按钮区域 */}
        <div className="btn-group">
          {/* 生成按钮 */}
          <button
            className="btn btn-primary"
            onClick={handleGenerate}
            disabled={isGenerating}
          >
            {isGenerating ? "生成中..." : "生成 MP3"}
          </button>
        </div>

        {/* 音频预览播放器（生成成功后显示） */}
        {audioUrl && (
          <div className="audio-player">
            <audio controls src={audioUrl} />
          </div>
        )}

        {/* 保存按钮（生成成功后显示） */}
        {mp3Path && (
          <button className="btn btn-secondary" onClick={handleSave}>
            保存 MP3
          </button>
        )}

        {/* 状态提示 */}
        {status && (
          <div className={`status ${getStatusClass()}`}>
            {status}
          </div>
        )}
      </div>

      {/* 页脚 */}
      <div className="footer">
        使用 Windows 系统自带语音 · 内置 ffmpeg 转 MP3
      </div>
    </div>
  );
}

// 导出组件
export default App;
