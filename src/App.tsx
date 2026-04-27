import { useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

const MAX_TEXT_LENGTH = 5000;

function App() {
  const [text, setText] = useState("");
  const [rate, setRate] = useState(0);
  const [volume, setVolume] = useState(100);
  const [status, setStatus] = useState("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [mp3Path, setMp3Path] = useState("");
  const [audioUrl, setAudioUrl] = useState("");
  const audioRef = useRef<HTMLAudioElement>(null);

  const handleGenerate = async () => {
    if (!text.trim()) {
      setStatus("请输入要转换的文字");
      return;
    }

    if (text.length > MAX_TEXT_LENGTH) {
      setStatus(`文字过长，最多 ${MAX_TEXT_LENGTH} 字`);
      return;
    }

    setIsGenerating(true);
    setStatus("正在生成语音...");

    try {
      const result = await invoke<string>("generate_speech", {
        text: text,
        rate: rate,
        volume: volume,
      });

      setMp3Path(result);

      // 使用 base64 读取音频，避免 convertFileSrc 权限问题
      const base64 = await invoke<string>("read_audio_base64", {
        path: result,
      });
      setAudioUrl(`data:audio/mpeg;base64,${base64}`);

      setStatus("语音生成成功！");
    } catch (err) {
      setStatus(`生成失败: ${err}`);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleSave = async () => {
    if (!mp3Path) return;

    try {
      const targetPath = await save({
        defaultPath: "output.mp3",
        filters: [{ name: "MP3", extensions: ["mp3"] }],
      });

      if (targetPath) {
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

  return (
    <div className="container">
      <header>
        <h1>🎙️ Windows 本地 TTS</h1>
        <p>完全本地运行，不调用任何云接口</p>
      </header>

      <div className="card">
        <div className="form-group">
          <label htmlFor="text">输入文字</label>
          <textarea
            id="text"
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="请输入要转换为语音的文字..."
            maxLength={MAX_TEXT_LENGTH}
          />
          <div className="chars">{text.length} / {MAX_TEXT_LENGTH}</div>
        </div>

        <div className="form-group">
          <label>语速: {rate}（范围 -10 到 10）</label>
          <div className="range-group">
            <span>-10</span>
            <input
              type="range"
              min="-10"
              max="10"
              step="1"
              value={rate}
              onChange={(e) => setRate(Number(e.target.value))}
            />
            <span>10</span>
          </div>
        </div>

        <div className="form-group">
          <label>音量: {volume}%（范围 0 到 100）</label>
          <div className="range-group">
            <span>0</span>
            <input
              type="range"
              min="0"
              max="100"
              step="1"
              value={volume}
              onChange={(e) => setVolume(Number(e.target.value))}
            />
            <span>100</span>
          </div>
        </div>

        <button
          className="btn"
          onClick={handleGenerate}
          disabled={isGenerating}
        >
          {isGenerating ? "生成中..." : "生成语音"}
        </button>

        {audioUrl && (
          <>
            <div className="audio-player">
              <audio ref={audioRef} controls src={audioUrl} />
            </div>
            <button className="btn btn-secondary" onClick={handleSave}>
              保存 MP3
            </button>
          </>
        )}

        {status && (
          <div
            className={`status ${
              status.includes("失败") || status.includes("过长")
                ? "error"
                : status.includes("成功")
                ? "success"
                : "info"
            }`}
          >
            {status}
          </div>
        )}
      </div>

      <div className="footer">
        <p>使用 Windows 系统自带语音 · 内置 ffmpeg 转 MP3</p>
      </div>
    </div>
  );
}

export default App;