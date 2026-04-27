import { useState } from "react";
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

  const handleClear = () => {
    setText("");
    setMp3Path("");
    setAudioUrl("");
    setStatus("");
  };

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
    setStatus("");
    setMp3Path("");
    setAudioUrl("");

    try {
      const result = await invoke<string>("generate_speech", {
        text: text,
        rate: rate,
        volume: volume,
      });

      setMp3Path(result);

      const base64 = await invoke<string>("read_audio_base64", {
        path: result,
      });
      setAudioUrl(`data:audio/mpeg;base64,${base64}`);

      setStatus("语音生成成功，可以试听或保存 MP3");
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

  const getStatusClass = () => {
    if (status.includes("成功")) return "status-success";
    if (status.includes("失败")) return "status-error";
    if (isGenerating || status.includes("生成中")) return "status-info";
    return "status-info";
  };

  return (
    <div className="container">
      <div className="card">
        <div className="header">
          <h1>Windows 本地文字转语音</h1>
          <p>完全离线 · 使用系统语音 · 本地生成 MP3</p>
        </div>

        <div className="form-group">
          <div className="form-label">
            <span className="form-label-text">输入文字</span>
          </div>
          <div className="clear-row">
            <button className="btn-clear" onClick={handleClear}>清空</button>
          </div>
          <textarea
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder="请输入要转换为语音的文字..."
            maxLength={MAX_TEXT_LENGTH}
          />
          <div className="chars-count">{text.length} / {MAX_TEXT_LENGTH}</div>
        </div>

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

        <div className="btn-group">
          <button
            className="btn btn-primary"
            onClick={handleGenerate}
            disabled={isGenerating}
          >
            {isGenerating ? "生成中..." : "生成 MP3"}
          </button>
        </div>

        {audioUrl && (
          <div className="audio-player">
            <audio controls src={audioUrl} />
          </div>
        )}

        {mp3Path && (
          <button className="btn btn-secondary" onClick={handleSave}>
            保存 MP3
          </button>
        )}

        {status && (
          <div className={`status ${getStatusClass()}`}>
            {status}
          </div>
        )}
      </div>

      <div className="footer">
        使用 Windows 系统自带语音 · 内置 ffmpeg 转 MP3
      </div>
    </div>
  );
}

export default App;